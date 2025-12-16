use pinocchio::account_info::AccountInfo;
use pinocchio::instruction::{Seed, Signer};
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::try_find_program_address;
use pinocchio::ProgramResult;

use pinocchio_log::log;

use crate::error::UniPinoNftErr;
use super::*;

pub struct InitPlatform<'a> {
    pub authority: &'a AccountInfo,
    pub platform_pda: &'a AccountInfo,
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
            log!("[ERROR] address {} is not the correct signer", authority.key());
            return Err(ProgramError::InvalidAccountOwner);
        }

        let (pda, bump) = try_find_program_address(
            &[b"administer", authority.key().as_ref()],
            &ID
        ).ok_or(UniPinoNftErr::PdaErr).map_err(|e| ProgramError::from(e))?;

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

