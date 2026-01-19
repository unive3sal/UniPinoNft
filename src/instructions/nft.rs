use alloc::string::ToString;
use bytemuck::{bytes_of, try_from_bytes};
use pinocchio::account_info::AccountInfo;
use pinocchio::instruction::{Seed, Signer};
use pinocchio::program_error::ProgramError;
use pinocchio::sysvars::Sysvar;
use pinocchio::sysvars::rent::Rent;
use pinocchio::{ProgramResult, pubkey::try_find_program_address};
use pinocchio_log::log;
use pinocchio_system::instructions::{CreateAccount, Transfer};
use pinocchio_token_2022::state::Mint;
use pinocchio_token_2022::{
    ID as TOKEN_2022_ID,
    instructions::{CloseAccount, InitializeMint2},
};

use crate::error::UniPinoNftErr;
use crate::state::nft_meta::NftMeta;
use crate::state::platform::Platform;
use crate::state::user::User;

use super::*;

pub struct MintNft<'a> {
    pub administrator: &'a AccountInfo,
    pub platform_pda: &'a AccountInfo,
    pub user_pda: &'a AccountInfo,
    pub mint_pda: &'a AccountInfo,
    pub metadata_pda: &'a AccountInfo,
    pub fee_receiver: &'a AccountInfo,
    pub mint_nft_args: &'a MintNftArgs,
}

impl<'a> MintNft<'a> {
    pub const DISCRIMINATOR: &'a u8 = &3;

    pub fn process(self) -> ProgramResult {
        if !self.administrator.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !self.platform_pda.is_owned_by(&ID)
            || !self.user_pda.is_owned_by(&ID)
            || self.platform_pda.lamports() == 0
            || self.user_pda.lamports() == 0
        {
            return Err(UniPinoNftErr::UninitPda.into());
        }

        let mut platform_data_bytes = self.platform_pda.try_borrow_mut_data()?;
        let platform = Platform::try_from_bytes_mut(platform_data_bytes.as_mut())?;

        let mut user_data_bytes = self.user_pda.try_borrow_mut_data()?;
        let user = User::try_from_bytes_mut(user_data_bytes.as_mut())?;

        if platform.administrator != self.administrator.key().as_ref()
            || user.owner != self.platform_pda.key().as_ref()
        {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Collect mint fee if configured
        if platform.mint_fee > 0 {
            // Validate fee_receiver matches platform configuration
            if platform.fee_receiver != self.fee_receiver.key().as_ref() {
                return Err(ProgramError::InvalidAccountOwner);
            }

            Transfer {
                from: self.administrator,
                to: self.fee_receiver,
                lamports: platform.mint_fee,
            }
            .invoke()?;

            log!("collected mint fee: {} lamports", platform.mint_fee);
        }

        let mint_pda_seeds = [
            &self.mint_nft_args.user_uuid.to_le_bytes(),
            self.user_pda.key().as_ref(),
            &TOKEN_2022_ID,
        ];
        let (mint_pda, mint_bump) = try_find_program_address(&mint_pda_seeds, &TOKEN_2022_ID)
            .ok_or(UniPinoNftErr::PdaErr)?;

        if mint_pda != self.mint_pda.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }

        let platform_seeds = [
            Seed::from(platform::PLATFORM_TOKEN),
            Seed::from(self.administrator.key().as_ref()),
            Seed::from(core::slice::from_ref(&platform.bump)),
        ];
        let platform_signer = Signer::from(&platform_seeds);

        let user_seeds = [
            Seed::from(user::USER_TOKEN),
            Seed::from(self.platform_pda.key().as_ref()),
            Seed::from(core::slice::from_ref(&user.bump)),
        ];
        let user_signer = Signer::from(&user_seeds);

        let mint_lamports = Rent::get()?.minimum_balance(Mint::BASE_LEN);
        if self.mint_pda.lamports() == 0 {
            CreateAccount {
                from: self.administrator,
                to: self.mint_pda,
                lamports: mint_lamports,
                space: Mint::BASE_LEN as u64,
                owner: &TOKEN_2022_ID,
            }
            .invoke_signed(&[platform_signer, user_signer])?
        }

        let binding = [
            Seed::from(&mint_pda),
            Seed::from(core::slice::from_ref(&mint_bump)),
        ];
        let mint_signer = Signer::from(&binding);

        InitializeMint2 {
            mint: self.mint_pda,
            decimals: 0,
            mint_authority: self.user_pda.key(),
            freeze_authority: None,
            token_program: &TOKEN_2022_ID,
        }
        .invoke()?;

        let metadata_pda_seeds = [b"metadata", self.mint_pda.key().as_ref(), &TOKEN_2022_ID];
        let (metadata_pda, _) = try_find_program_address(&metadata_pda_seeds, &TOKEN_2022_ID)
            .ok_or(UniPinoNftErr::PdaErr)?;

        if metadata_pda != self.metadata_pda.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }

        let metadata_lamport = Rent::get()?.minimum_balance(NftMeta::INIT_SPACE);
        if self.metadata_pda.lamports() == 0 {
            let platform_seeds = [
                Seed::from(platform::PLATFORM_TOKEN),
                Seed::from(self.administrator.key().as_ref()),
                Seed::from(core::slice::from_ref(&platform.bump)),
            ];
            let platform_signer = Signer::from(&platform_seeds);

            let user_seeds = [
                Seed::from(user::USER_TOKEN),
                Seed::from(self.platform_pda.key().as_ref()),
                Seed::from(core::slice::from_ref(&user.bump)),
            ];
            let user_signer = Signer::from(&user_seeds);

            CreateAccount {
                from: self.administrator,
                to: self.metadata_pda,
                lamports: metadata_lamport,
                space: NftMeta::INIT_SPACE as u64,
                owner: &ID,
            }
            .invoke_signed(&[platform_signer, user_signer, mint_signer])?;
        }
        let metadata = NftMeta {
            discriminator: NftMeta::DISCRIMINATOR,
            name: self.mint_nft_args.asset_name,
            collection: [0; 64],
            uri: self.mint_nft_args.uri,
            description: self.mint_nft_args.desc,
        };
        self.metadata_pda
            .try_borrow_mut_data()?
            .copy_from_slice(bytes_of(&metadata));

        // update mint count
        platform.total_mints = platform
            .total_mints
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        user.nft_count = user
            .nft_count
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        log!("mint nft success");
        Ok(())
    }
}

impl<'a> TryFrom<(&'a [AccountInfo], &'a [u8])> for MintNft<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountInfo], &'a [u8])) -> Result<Self, Self::Error> {
        let (accounts, instruction_data) = value;

        let [
            administrator,
            platform_pda,
            user_pda,
            mint_pda,
            metadata_pda,
            fee_receiver,
            _,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if instruction_data.len() != size_of::<MintNftArgs>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mint_nft_args = try_from_bytes::<MintNftArgs>(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self {
            administrator,
            platform_pda,
            user_pda,
            mint_pda,
            metadata_pda,
            fee_receiver,
            mint_nft_args,
        })
    }
}

pub struct UpdateNFTMetadata<'a> {
    pub administrator: &'a AccountInfo,
    pub platform_pda: &'a AccountInfo,
    pub user_pda: &'a AccountInfo,
    pub mint_pda: &'a AccountInfo,
    pub metadata_pda: &'a AccountInfo,
    pub nft_meta: &'a super::NftMeta,
}

impl<'a> UpdateNFTMetadata<'a> {
    pub const DISCRIMINATOR: &'a u8 = &4;

    pub fn process(self) -> ProgramResult {
        if !self.administrator.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !self.platform_pda.is_owned_by(&ID)
            || !self.user_pda.is_owned_by(&ID)
            || !self.metadata_pda.is_owned_by(&ID)
            || self.platform_pda.lamports() == 0
            || self.user_pda.lamports() == 0
            || self.metadata_pda.lamports() == 0
        {
            return Err(UniPinoNftErr::UninitPda.into());
        }

        let platform_data_bytes = self.platform_pda.try_borrow_data()?;
        let platform = Platform::try_from_bytes(platform_data_bytes.as_ref())?;

        let user_data_bytes = self.user_pda.try_borrow_data()?;
        let user = User::try_from_bytes(user_data_bytes.as_ref())?;

        if platform.administrator != self.administrator.key().as_ref()
            || user.owner != self.platform_pda.key().as_ref()
        {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let metadata_pda_seeds = [b"metadata", self.mint_pda.key().as_ref(), &TOKEN_2022_ID];
        let (metadata_pda, _) = try_find_program_address(&metadata_pda_seeds, &TOKEN_2022_ID)
            .ok_or(UniPinoNftErr::PdaErr)?;

        if metadata_pda != self.metadata_pda.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }

        let updated_metadata = NftMeta {
            discriminator: NftMeta::DISCRIMINATOR,
            name: self.nft_meta.name,
            collection: self.nft_meta.collection,
            uri: self.nft_meta.uri,
            description: self.nft_meta.description,
        };
        self.metadata_pda
            .try_borrow_mut_data()?
            .copy_from_slice(bytes_of(&updated_metadata));

        log!("nft metadata updated");
        Ok(())
    }
}

impl<'a> TryFrom<(&'a [AccountInfo], &'a [u8])> for UpdateNFTMetadata<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountInfo], &'a [u8])) -> Result<Self, Self::Error> {
        let (accounts, instruction_data) = value;

        let [
            administrator,
            platform_pda,
            user_pda,
            mint_pda,
            metadata_pda,
            _,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if instruction_data.len() != size_of::<super::NftMeta>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let nft_meta = try_from_bytes::<super::NftMeta>(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self {
            administrator,
            platform_pda,
            user_pda,
            mint_pda,
            metadata_pda,
            nft_meta,
        })
    }
}

pub struct BurnNft<'a> {
    pub administrator: &'a AccountInfo,
    pub platform_pda: &'a AccountInfo,
    pub user_pda: &'a AccountInfo,
    pub mint_pda: &'a AccountInfo,
    pub metadata_pda: &'a AccountInfo,
}

impl<'a> BurnNft<'a> {
    pub const DISCRIMINATOR: &'a u8 = &5;

    pub fn process(self) -> ProgramResult {
        if !self.administrator.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !self.platform_pda.is_owned_by(&ID)
            || !self.user_pda.is_owned_by(&ID)
            || !self.metadata_pda.is_owned_by(&ID)
            || self.platform_pda.lamports() == 0
            || self.user_pda.lamports() == 0
            || self.metadata_pda.lamports() == 0
        {
            return Err(UniPinoNftErr::UninitPda.into());
        }

        let mut platform_data_bytes = self.platform_pda.try_borrow_mut_data()?;
        let platform = Platform::try_from_bytes_mut(platform_data_bytes.as_mut())?;

        let mut user_data_bytes = self.user_pda.try_borrow_mut_data()?;
        let user = User::try_from_bytes_mut(user_data_bytes.as_mut())?;

        if platform.administrator != self.administrator.key().as_ref()
            || user.owner != self.platform_pda.key().as_ref()
        {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Validate metadata PDA is derived from the mint
        let metadata_pda_seeds = [b"metadata", self.mint_pda.key().as_ref(), &TOKEN_2022_ID];
        let (metadata_pda, _) = try_find_program_address(&metadata_pda_seeds, &TOKEN_2022_ID)
            .ok_or(UniPinoNftErr::PdaErr)?;

        if metadata_pda != self.metadata_pda.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }

        // Close the mint account via Token-2022 CPI
        // The user_pda is the mint authority, so we need it to sign
        let user_uuid_val = { user.user_uuid }; // Copy from packed struct
        let user_uuid_str = user_uuid_val.to_string();
        let user_seeds = [
            Seed::from(user::USER_TOKEN),
            Seed::from(user_uuid_str.as_bytes()),
            Seed::from(self.platform_pda.key().as_ref()),
            Seed::from(core::slice::from_ref(&platform.bump)),
            Seed::from(core::slice::from_ref(&user.bump)),
        ];
        let user_signer = Signer::from(&user_seeds);

        CloseAccount {
            account: self.mint_pda,
            destination: self.administrator,
            authority: self.user_pda,
            token_program: &TOKEN_2022_ID,
        }
        .invoke_signed(&[user_signer])?;

        // Close metadata account by transferring lamports to administrator
        let metadata_lamports = self.metadata_pda.lamports();
        unsafe {
            *self.metadata_pda.borrow_mut_lamports_unchecked() = 0;
            *self.administrator.borrow_mut_lamports_unchecked() = self
                .administrator
                .lamports()
                .checked_add(metadata_lamports)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        }

        // Zero out the metadata account data
        self.metadata_pda.try_borrow_mut_data()?.fill(0);

        // Update mint counts
        platform.total_mints = platform
            .total_mints
            .checked_sub(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        user.nft_count = user
            .nft_count
            .checked_sub(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        log!("burn nft success");
        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo]> for BurnNft<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [
            administrator,
            platform_pda,
            user_pda,
            mint_pda,
            metadata_pda,
            _,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self {
            administrator,
            platform_pda,
            user_pda,
            mint_pda,
            metadata_pda,
        })
    }
}
