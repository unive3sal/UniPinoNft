use bytemuck::try_from_bytes;
use pinocchio::error::ProgramError;
use pinocchio::{AccountView, ProgramResult};

use super::*;

pub struct MintNft<'a> {
    pub administrator: &'a AccountView,
    pub platform_pda: &'a AccountView,
    pub user_pda: &'a AccountView,
    pub mint_pda: &'a AccountView,
    pub metadata_pda: &'a AccountView,
    pub fee_receiver: &'a AccountView,
    pub mint_nft_args: &'a MintNftArgs,
}

impl<'a> MintNft<'a> {
    pub const DISCRIMINATOR: &'a u8 = &3;

    pub fn process(self) -> ProgramResult {
        validate_mint_preconditions(self.administrator.is_signer())
    }
}

fn validate_mint_preconditions(administrator_is_signer: bool) -> ProgramResult {
    if !administrator_is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    Ok(())
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for MintNft<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
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

        if instruction_data.len() != core::mem::size_of::<MintNftArgs>() {
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
    pub administrator: &'a AccountView,
    pub platform_pda: &'a AccountView,
    pub user_pda: &'a AccountView,
    pub mint_pda: &'a AccountView,
    pub metadata_pda: &'a AccountView,
    pub nft_meta: &'a super::NftMeta,
}

impl<'a> UpdateNFTMetadata<'a> {
    pub const DISCRIMINATOR: &'a u8 = &4;

    pub fn process(self) -> ProgramResult {
        let _ = self;
        Err(ProgramError::InvalidInstructionData)
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for UpdateNFTMetadata<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
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

        if instruction_data.len() != core::mem::size_of::<super::NftMeta>() {
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
    pub administrator: &'a AccountView,
    pub platform_pda: &'a AccountView,
    pub user_pda: &'a AccountView,
    pub mint_pda: &'a AccountView,
    pub metadata_pda: &'a AccountView,
}

impl<'a> BurnNft<'a> {
    pub const DISCRIMINATOR: &'a u8 = &5;

    pub fn process(self) -> ProgramResult {
        let _ = self;
        Err(ProgramError::InvalidInstructionData)
    }
}

impl<'a> TryFrom<&'a [AccountView]> for BurnNft<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mint_requires_admin_signature() {
        let result = validate_mint_preconditions(false);
        assert_eq!(result, Err(ProgramError::MissingRequiredSignature));
    }
}
