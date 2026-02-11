#![no_std]
#![allow(non_snake_case)]
#![allow(unexpected_cfgs)]

extern crate alloc;

mod error;
mod instructions;
mod state;

use pinocchio::error::ProgramError;
use pinocchio::{
    default_allocator, nostd_panic_handler, program_entrypoint, Address, ProgramResult,
};
use solana_address::declare_id;

use crate::instructions::{nft::*, platform::*, user::*};

declare_id!("6jpuWYTM3ARc5CHrMBtR1c7gyjkMTsJoYT7PqqhMpRWh");

program_entrypoint!(process_instruction);
nostd_panic_handler!();
default_allocator!();

pub fn process_instruction(
    _program_id: &Address,
    accounts: &[pinocchio::AccountView],
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
