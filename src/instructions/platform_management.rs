use bytemuck::bytes_of;
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::Sysvar;
use pinocchio::ProgramResult;
use pinocchio::account_info::AccountInfo;
use pinocchio::instruction::{Seed, Signer};
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::try_find_program_address;

use pinocchio_log::log;
use pinocchio_system::instructions::CreateAccount;

use super::*;
use crate::error::UniPinoNftErr;
use crate::state::platform_state::PlatformState;

pub struct InitPlatform<'a> {
    authority: &'a AccountInfo,
    platform_pda: &'a AccountInfo,
}

pub struct UpdatePlatformConfig<'a> {
    authority: &'a AccountInfo,
    platform_pda: &'a AccountInfo,
}

impl<'a> InitPlatform<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(self) -> ProgramResult {
        let InitPlatform {
            authority,
            platform_pda,
        } = self;

        // authority is the administration wallet which is configured by server backend
        if !authority.is_signer() {
            log!(
                "[ERROR] address {} is not the correct signer",
                authority.key()
            );
            return Err(ProgramError::InvalidAccountOwner);
        }

        let (pda, bump) = try_find_program_address(&[b"administer", authority.key().as_ref()], &ID)
            .ok_or(UniPinoNftErr::PdaErr)
            .map_err(|e| ProgramError::from(e))?;

        if pda != platform_pda.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }

        if platform_pda.lamports() > 0 {
            return Err(ProgramError::from(UniPinoNftErr::ReInitPda));
        }

        let signer_seeds = [
            Seed::from(b"administer".as_slice()),
            Seed::from(authority.key().as_ref()),
            Seed::from(core::slice::from_ref(&bump)),
        ];
        let signer = Signer::from(&signer_seeds);

        // calculate rents
        let min_lamports = Rent::get()?.minimum_balance(PlatformState::INIT_SPACE);
        log!("Init platform PDA requires min balance: {}", min_lamports);

        CreateAccount {
            from: &authority,
            to: &platform_pda,
            lamports: min_lamports,
            space: PlatformState::INIT_SPACE as u64,
            owner: &ID,
        }
        .invoke_signed(&[signer])?;

        // fill data field in PDA
        let platform_init_state = PlatformState::new(*authority.key(), *authority.key(), bump);
        platform_pda.try_borrow_mut_data()?
            .copy_from_slice(bytes_of(&platform_init_state));

        if platform_pda.try_borrow_data()?[0..8] != platform_init_state.discriminator {
            return Err(ProgramError::from(UniPinoNftErr::InitPlatformPdaErr));
        }

        log!("platform pda created");
        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitPlatform<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        if accounts.len() < 2 {
            log!("[Error] accounts len: {}", accounts.len());
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let signer = &accounts[0];
        let platform_pda = &accounts[1];

        Ok(Self {
            authority: signer,
            platform_pda: platform_pda,
        })
    }
}

impl<'a> UpdatePlatformConfig<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;
    
    fn process(self) -> ProgramResult {
        
    }
}

impl<'a> TryFrom<(&'a [AccountInfo], &'a [u8])> for UpdatePlatformConfig {
    
}
