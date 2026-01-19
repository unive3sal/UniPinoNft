use bytemuck::{bytes_of, try_from_bytes};
use pinocchio::ProgramResult;
use pinocchio::account_info::AccountInfo;
use pinocchio::instruction::{Seed, Signer};
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::try_find_program_address;
use pinocchio::sysvars::Sysvar;
use pinocchio::sysvars::rent::Rent;

use pinocchio_log::log;
use pinocchio_system::instructions::CreateAccount;

use super::*;
use crate::error::UniPinoNftErr;
use crate::state::platform::Platform;

pub const PLATFORM_TOKEN: &[u8] = b"administer";

pub struct InitPlatform<'a> {
    administrator: &'a AccountInfo,
    platform_pda: &'a AccountInfo,
}

pub struct UpdatePlatformConfig<'a> {
    administrator: &'a AccountInfo,
    platform_pda: &'a AccountInfo,
    update_args: &'a UpdatePlatformArgs,
}

impl<'a> InitPlatform<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(self) -> ProgramResult {
        // authority is the administration wallet which is configured by server backend
        if !self.administrator.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (pda, bump) =
            try_find_program_address(&[PLATFORM_TOKEN, self.administrator.key().as_ref()], &ID)
                .ok_or(UniPinoNftErr::PdaErr)?;

        if pda != self.platform_pda.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }

        if self.platform_pda.lamports() > 0 {
            return Err(UniPinoNftErr::ReInitPda.into());
        }

        let signer_seeds = [
            Seed::from(PLATFORM_TOKEN),
            Seed::from(self.administrator.key().as_ref()),
            Seed::from(core::slice::from_ref(&bump)),
        ];
        let signer = Signer::from(&signer_seeds);

        let min_lamports = Rent::get()?.minimum_balance(Platform::INIT_SPACE);
        log!("Init platform PDA requires min balance: {}", min_lamports);

        CreateAccount {
            from: &self.administrator,
            to: &self.platform_pda,
            lamports: min_lamports,
            space: Platform::INIT_SPACE as u64,
            owner: &ID,
        }
        .invoke_signed(&[signer])?;

        // fill data field in PDA
        let platform_init_state = Platform::new(*self.administrator.key(), bump);
        self.platform_pda
            .try_borrow_mut_data()?
            .copy_from_slice(bytes_of(&platform_init_state));

        log!("platform pda created");
        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitPlatform<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [administrator, platform_pda, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self {
            administrator: administrator,
            platform_pda: platform_pda,
        })
    }
}

impl<'a> UpdatePlatformConfig<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(self) -> ProgramResult {
        if !self.administrator.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !self.platform_pda.is_owned_by(&ID) || self.platform_pda.lamports() == 0 {
            return Err(UniPinoNftErr::UninitPda.into());
        }

        let mut platform_data_bytes = self.platform_pda.try_borrow_mut_data()?;
        let platform_state = Platform::try_from_bytes_mut(platform_data_bytes.as_mut())?;

        if platform_state.administrator != self.administrator.key().as_ref() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let (pda, bump) =
            try_find_program_address(&[PLATFORM_TOKEN, self.administrator.key().as_ref()], &ID)
                .ok_or(UniPinoNftErr::PdaErr)?;

        if pda != self.platform_pda.key().as_ref() || bump != platform_state.bump {
            return Err(ProgramError::InvalidSeeds);
        }

        // update plateform state
        if self.update_args.is_receiver_valid != 0 {
            log!("update fee_receiver to {}", &self.update_args.fee_receiver);
            platform_state
                .fee_receiver
                .copy_from_slice(&self.update_args.fee_receiver);
        }
        platform_state.mint_fee = self.update_args.mint_fee;

        log!("platform config updated");
        Ok(())
    }
}

impl<'a> TryFrom<(&'a [AccountInfo], &'a [u8])> for UpdatePlatformConfig<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountInfo], &'a [u8])) -> Result<Self, Self::Error> {
        let (accounts, instruction_data) = value;

        let [administrator, platform_pda, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if instruction_data.len() != size_of::<UpdatePlatformArgs>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let update_platform_args = try_from_bytes::<UpdatePlatformArgs>(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self {
            administrator: administrator,
            platform_pda: platform_pda,
            update_args: &update_platform_args,
        })
    }
}
