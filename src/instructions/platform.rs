use bytemuck::{bytes_of, try_from_bytes};
use pinocchio::cpi::{Seed, Signer};
use pinocchio::error::ProgramError;
use pinocchio::sysvars::{Sysvar, rent::Rent};
use pinocchio::{AccountView, ProgramResult};
use pinocchio_log::log;
use pinocchio_system::instructions::CreateAccount;
use solana_address::Address;

use super::*;
use crate::ID;
use crate::error::UniPinoNftErr;
use crate::state::platform::Platform;

pub const PLATFORM_TOKEN: &[u8] = b"administer";

pub struct InitPlatform<'a> {
    administrator: &'a AccountView,
    platform_pda: &'a AccountView,
}

pub struct UpdatePlatformConfig<'a> {
    administrator: &'a AccountView,
    platform_pda: &'a AccountView,
    update_args: &'a UpdatePlatformArgs,
}

impl<'a> InitPlatform<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(self) -> ProgramResult {
        if !self.administrator.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (pda, bump) = Address::find_program_address(
            &[PLATFORM_TOKEN, self.administrator.address().as_ref()],
            &ID,
        );

        if pda != *self.platform_pda.address() {
            return Err(ProgramError::InvalidSeeds);
        }

        if self.platform_pda.lamports() > 0 {
            return Err(UniPinoNftErr::ReInitPda.into());
        }

        let signer_seeds = [
            Seed::from(PLATFORM_TOKEN),
            Seed::from(self.administrator.address().as_ref()),
            Seed::from(core::slice::from_ref(&bump)),
        ];
        let signer = Signer::from(&signer_seeds);

        let min_lamports = Rent::get()?.try_minimum_balance(Platform::INIT_SPACE)?;

        CreateAccount {
            from: self.administrator,
            to: self.platform_pda,
            lamports: min_lamports,
            space: Platform::INIT_SPACE as u64,
            owner: &ID,
        }
        .invoke_signed(&[signer])?;

        let platform_init_state = Platform::new(*self.administrator.address().as_array(), bump);
        self.platform_pda
            .try_borrow_mut()?
            .copy_from_slice(bytes_of(&platform_init_state));

        log!("platform initialized");
        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountView]> for InitPlatform<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [administrator, platform_pda, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self {
            administrator,
            platform_pda,
        })
    }
}

impl<'a> UpdatePlatformConfig<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(self) -> ProgramResult {
        if !self.administrator.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !self.platform_pda.owned_by(&ID) || self.platform_pda.lamports() == 0 {
            return Err(UniPinoNftErr::UninitPda.into());
        }

        let mut platform_data = self.platform_pda.try_borrow_mut()?;
        let platform_state = Platform::try_from_bytes_mut(platform_data.as_mut())?;

        if platform_state.administrator != *self.administrator.address().as_array() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let (pda, bump) = Address::find_program_address(
            &[PLATFORM_TOKEN, self.administrator.address().as_ref()],
            &ID,
        );
        if pda != *self.platform_pda.address() || bump != platform_state.bump {
            return Err(ProgramError::InvalidSeeds);
        }

        if self.update_args.is_receiver_valid != 0 {
            platform_state.fee_receiver = self.update_args.fee_receiver;
        }
        platform_state.mint_fee = self.update_args.mint_fee;

        log!("platform updated");
        Ok(())
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for UpdatePlatformConfig<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let (accounts, instruction_data) = value;

        let [administrator, platform_pda, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if instruction_data.len() != core::mem::size_of::<UpdatePlatformArgs>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let update_platform_args = try_from_bytes::<UpdatePlatformArgs>(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self {
            administrator,
            platform_pda,
            update_args: update_platform_args,
        })
    }
}
