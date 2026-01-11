use bytemuck::{bytes_of, try_from_bytes, try_from_bytes_mut};
use pinocchio::account_info::AccountInfo;
use pinocchio::instruction::{Seed, Signer};
use pinocchio::program_error::ProgramError;
use pinocchio::sysvars::Sysvar;
use pinocchio::sysvars::rent::Rent;
use pinocchio::{ProgramResult, pubkey::try_find_program_address};
use pinocchio_log::log;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token_2022::state::Mint;
use pinocchio_token_2022::{ID as TOKEN_2022_ID, instructions::InitializeMint2};

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
        let platform = try_from_bytes_mut::<Platform>(platform_data_bytes.as_mut())
            .map_err(|_| ProgramError::AccountBorrowFailed)?;

        let mut user_data_bytes = self.user_pda.try_borrow_mut_data()?;
        let user = try_from_bytes_mut::<User>(user_data_bytes.as_mut())
            .map_err(|_| ProgramError::AccountBorrowFailed)?;

        if platform.administrator != self.administrator.key().as_ref()
            || user.owner != self.platform_pda.key().as_ref()
        {
            return Err(ProgramError::InvalidAccountOwner);
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
            name: self.mint_nft_args.asset_name,
            collection: [0; 64],
            uri: self.mint_nft_args.uri,
            description: self.mint_nft_args.desc,
        };
        self.metadata_pda
            .try_borrow_mut_data()?
            .copy_from_slice(bytes_of(&metadata));

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
            administrator: administrator,
            platform_pda: platform_pda,
            user_pda: user_pda,
            mint_pda: mint_pda,
            metadata_pda: metadata_pda,
            mint_nft_args: mint_nft_args,
        })
    }
}
