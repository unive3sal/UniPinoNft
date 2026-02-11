use pinocchio::error::ProgramError;
use pinocchio::{AccountView, ProgramResult};

use super::*;

pub struct MintNft<'a> {
    pub authority: &'a AccountView,
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
        let Self {
            authority,
            platform_pda,
            user_pda,
            mint_pda,
            metadata_pda,
            fee_receiver,
            mint_nft_args,
        } = self;

        let _ = (
            platform_pda,
            user_pda,
            mint_pda,
            metadata_pda,
            fee_receiver,
            mint_nft_args,
        );

        validate_mint_preconditions(authority.is_signer())
    }
}

fn validate_mint_preconditions(authority_is_signer: bool) -> ProgramResult {
    if !authority_is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    Ok(())
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for MintNft<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let (accounts, instruction_data) = value;

        let [authority, platform_pda, user_pda, mint_pda, metadata_pda, fee_receiver, _] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let mint_nft_args = parse_instruction_data::<MintNftArgs>(instruction_data)?;

        Ok(Self {
            authority,
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
    pub authority: &'a AccountView,
    pub platform_pda: &'a AccountView,
    pub user_pda: &'a AccountView,
    pub mint_pda: &'a AccountView,
    pub metadata_pda: &'a AccountView,
    pub nft_meta: &'a super::NftMetaArgs,
}

impl<'a> UpdateNFTMetadata<'a> {
    pub const DISCRIMINATOR: &'a u8 = &4;

    pub fn process(self) -> ProgramResult {
        let Self {
            authority,
            platform_pda,
            user_pda,
            mint_pda,
            metadata_pda,
            nft_meta,
        } = self;
        let _ = (
            authority,
            platform_pda,
            user_pda,
            mint_pda,
            metadata_pda,
            nft_meta,
        );

        Err(ProgramError::InvalidInstructionData)
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for UpdateNFTMetadata<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let (accounts, instruction_data) = value;

        let [authority, platform_pda, user_pda, mint_pda, metadata_pda, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let nft_meta = parse_instruction_data::<super::NftMetaArgs>(instruction_data)?;

        Ok(Self {
            authority,
            platform_pda,
            user_pda,
            mint_pda,
            metadata_pda,
            nft_meta,
        })
    }
}

pub struct BurnNft<'a> {
    pub authority: &'a AccountView,
    pub platform_pda: &'a AccountView,
    pub user_pda: &'a AccountView,
    pub mint_pda: &'a AccountView,
    pub metadata_pda: &'a AccountView,
}

impl<'a> BurnNft<'a> {
    pub const DISCRIMINATOR: &'a u8 = &5;

    pub fn process(self) -> ProgramResult {
        let Self {
            authority,
            platform_pda,
            user_pda,
            mint_pda,
            metadata_pda,
        } = self;
        let _ = (authority, platform_pda, user_pda, mint_pda, metadata_pda);

        Err(ProgramError::InvalidInstructionData)
    }
}

impl<'a> TryFrom<&'a [AccountView]> for BurnNft<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, platform_pda, user_pda, mint_pda, metadata_pda, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self {
            authority,
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
