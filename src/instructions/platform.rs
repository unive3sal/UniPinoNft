use bytemuck::checked::try_from_bytes_mut;
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

const PLATFORM_TOKEN: &[u8] = b"administer";

pub struct InitPlatform<'a> {
    authority: &'a AccountInfo,
    platform_pda: &'a AccountInfo,
}

pub struct UpdatePlatformConfig<'a> {
    authority: &'a AccountInfo,
    platform_pda: &'a AccountInfo,
    update_args: UpdatePlatformArgs,
}

impl<'a> InitPlatform<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;
    pub const ACCOUNT_NUM: usize = 2;

    pub fn process(self) -> ProgramResult {
        // authority is the administration wallet which is configured by server backend
        if !self.authority.is_signer() {
            log!(
                "[ERROR] address {} is not the correct signer",
                self.authority.key()
            );
            return Err(ProgramError::InvalidAccountOwner);
        }

        let (pda, bump) =
            try_find_program_address(&[PLATFORM_TOKEN, self.authority.key().as_ref()], &ID)
                .ok_or(UniPinoNftErr::PdaErr)
                .map_err(|e| ProgramError::from(e))?;

        if pda != self.platform_pda.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }

        if self.platform_pda.lamports() > 0 {
            return Err(ProgramError::from(UniPinoNftErr::ReInitPda));
        }

        let signer_seeds = [
            Seed::from(PLATFORM_TOKEN),
            Seed::from(self.authority.key().as_ref()),
            Seed::from(core::slice::from_ref(&bump)),
        ];
        let signer = Signer::from(&signer_seeds);

        // calculate rents
        let min_lamports = Rent::get()?.minimum_balance(Platform::INIT_SPACE);
        log!("Init platform PDA requires min balance: {}", min_lamports);

        CreateAccount {
            from: &self.authority,
            to: &self.platform_pda,
            lamports: min_lamports,
            space: Platform::INIT_SPACE as u64,
            owner: &ID,
        }
        .invoke_signed(&[signer])?;

        // fill data field in PDA
        let platform_init_state = Platform::new(*self.authority.key(), *self.authority.key(), bump);
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
        if accounts.len() < Self::ACCOUNT_NUM {
            log!(
                "[Error] input accounts len: {}, require: {}",
                accounts.len(),
                Self::ACCOUNT_NUM
            );
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
    pub const ACCOUNT_NUM: usize = 2;

    pub fn process(self) -> ProgramResult {
        if !self.authority.is_signer() {
            log!(
                "[ERROR] address {} is not the correct signer",
                self.authority.key()
            );
            return Err(ProgramError::InvalidAccountOwner);
        }

        if self.platform_pda.lamports() == 0 {
            return Err(ProgramError::from(UniPinoNftErr::PlatformPdaUninit));
        }

        let mut platform_data_bytes = self
            .platform_pda
            .try_borrow_mut_data()
            .map_err(|_| ProgramError::from(UniPinoNftErr::PlatformPdaUninit))?;
        let platform_state = try_from_bytes_mut::<Platform>(platform_data_bytes.as_mut())
            .map_err(|_| ProgramError::from(UniPinoNftErr::PlatformPdaUninit))?;

        let (pda, bump) =
            try_find_program_address(&[PLATFORM_TOKEN, self.authority.key().as_ref()], &ID)
                .ok_or(UniPinoNftErr::PdaErr)
                .map_err(|e| ProgramError::from(e))?;

        if pda != self.platform_pda.key().as_ref() || bump != platform_state.bump {
            return Err(ProgramError::InvalidSeeds);
        }

        if self.platform_pda.is_owned_by(&ID)
            || platform_state.authority != self.authority.key().as_ref()
        {
            return Err(ProgramError::InvalidAccountOwner);
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

        if accounts.len() < Self::ACCOUNT_NUM {
            log!(
                "[Error] input accounts len: {}, require: {}",
                accounts.len(),
                Self::ACCOUNT_NUM
            );
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        if instruction_data.len() != size_of::<UpdatePlatformArgs>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let signer = &accounts[0];
        let platform_pda = &accounts[1];
        let update_platform_args = try_from_bytes::<UpdatePlatformArgs>(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self {
            authority: signer,
            platform_pda: platform_pda,
            update_args: *update_platform_args,
        })
    }
}
