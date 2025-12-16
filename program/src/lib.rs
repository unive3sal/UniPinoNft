#![no_std]
#![allow(non_snake_case)]

extern crate alloc;

mod error;
mod instructions;
mod state {
    pub mod platform_state;
}

#[cfg(feature = "bpf-entrypoint")]
mod entrypoint {
    use pinocchio::{
        entrypoint,
        account_info::AccountInfo,
        program_error::ProgramError,
        pubkey::Pubkey,
        ProgramResult
    };

    use crate::instructions::{
        platform_management::*
    };

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
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
