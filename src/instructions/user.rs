use alloc::string::ToString;

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
use crate::state::user::User;

pub const USER_TOKEN: &[u8] = b"user_wallet";

pub struct CreateUser<'a> {
    pub administrator: &'a AccountView,
    pub platform_pda: &'a AccountView,
    pub user_pda: &'a AccountView,
    pub user_uuid: &'a u128,
}

impl<'a> CreateUser<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

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

        if self.user_pda.lamports() > 0 {
            return Err(UniPinoNftErr::ReInitPda.into());
        }

        let user_uuid_seed = self.user_uuid.to_string();
        let (user_pda, user_bump) = Address::find_program_address(
            &[
                USER_TOKEN,
                user_uuid_seed.as_bytes(),
                self.platform_pda.address().as_ref(),
                core::slice::from_ref(&platform_state.bump),
            ],
            &ID,
        );

        if user_pda != *self.user_pda.address() {
            return Err(ProgramError::InvalidSeeds);
        }

        let min_lamports = Rent::get()?.try_minimum_balance(User::INIT_SPACE)?;

        let platform_seeds = [
            Seed::from(platform::PLATFORM_TOKEN),
            Seed::from(self.administrator.address().as_ref()),
            Seed::from(core::slice::from_ref(&platform_state.bump)),
        ];
        let platform_signer = Signer::from(&platform_seeds);

        let user_seeds = [
            Seed::from(USER_TOKEN),
            Seed::from(user_uuid_seed.as_bytes()),
            Seed::from(self.platform_pda.address().as_ref()),
            Seed::from(core::slice::from_ref(&platform_state.bump)),
            Seed::from(core::slice::from_ref(&user_bump)),
        ];
        let user_signer = Signer::from(&user_seeds);

        CreateAccount {
            from: self.administrator,
            to: self.user_pda,
            lamports: min_lamports,
            space: User::INIT_SPACE as u64,
            owner: &ID,
        }
        .invoke_signed(&[platform_signer, user_signer])?;

        let user_meta = User::new(
            *self.platform_pda.address().as_array(),
            *self.user_uuid,
            user_bump,
        );
        self.user_pda
            .try_borrow_mut()?
            .copy_from_slice(bytes_of(&user_meta));

        platform_state.total_users = platform_state
            .total_users
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        log!("user created");
        Ok(())
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for CreateUser<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let (accounts, instruction_data) = value;

        let [administrator, platform_pda, user_pda, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if instruction_data.len() != core::mem::size_of::<u128>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let user_uuid = try_from_bytes::<u128>(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self {
            administrator,
            platform_pda,
            user_pda,
            user_uuid,
        })
    }
}
