#![no_std]
#![allow(non_snake_case)]

extern crate alloc;

mod error;
mod instructions;
mod state {
    pub mod nft_meta;
    pub mod platform;
    pub mod user;
}

#[cfg(feature = "bpf-entrypoint")]
mod entrypoint {
    use pinocchio::{
        ProgramResult, account_info::AccountInfo, entrypoint, program_error::ProgramError,
        pubkey::Pubkey,
    };

    use crate::instructions::{nft::*, platform::*, user::*};

    use pinocchio_pubkey::declare_id;

    declare_id!("6jpuWYTM3ARc5CHrMBtR1c7gyjkMTsJoYT7PqqhMpRWh");

    entrypoint!(process_instruction);

    pub fn process_instruction(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        match instruction_data.split_first() {
            Some((InitPlatform::DISCRIMINATOR, _)) => InitPlatform::try_from(accounts)?.process(),
            Some((UpdatePlatformConfig::DISCRIMINATOR, data)) => {
                UpdatePlatformConfig::try_from((accounts, data))?.process()
            }
            Some((CreateUser::DISCRIMINATOR, data)) => {
                CreateUser::try_from((accounts, data))?.process()
            }
            Some((MintNft::DISCRIMINATOR, data)) => MintNft::try_from((accounts, data))?.process(),
            Some((UpdateNFTMetadata::DISCRIMINATOR, data)) => {
                UpdateNFTMetadata::try_from((accounts, data))?.process()
            }
            Some((BurnNft::DISCRIMINATOR, _)) => BurnNft::try_from(accounts)?.process(),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
