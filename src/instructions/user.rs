use alloc::string::ToString;
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

use crate::error::UniPinoNftErr;
use crate::state::platform::Platform;
use crate::state::user::User;

use super::*;

pub const USER_TOKEN: &[u8] = b"user_wallet";

pub struct CreateUser<'a> {
    pub administrator: &'a AccountInfo,
    pub platform_pda: &'a AccountInfo,
    pub user_pda: &'a AccountInfo,
    pub user_uuid: &'a u128,
}

impl<'a> CreateUser<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

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

        if self.user_pda.lamports() > 0 {
            return Err(UniPinoNftErr::ReInitPda.into());
        }

        let (user_pda, user_bump) = try_find_program_address(
            &[
                USER_TOKEN,
                self.user_uuid.to_string().as_bytes(),
                self.platform_pda.key().as_ref(),
                core::slice::from_ref(&platform_state.bump),
            ],
            &ID,
        )
        .ok_or(UniPinoNftErr::PdaErr)?;

        if user_pda != self.user_pda.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }

        let min_lamports = Rent::get()?.minimum_balance(User::INIT_SPACE);
        log!("Init user PDA requires min balance: {}", min_lamports);

        let platform_seeds = [
            Seed::from(platform::PLATFORM_TOKEN),
            Seed::from(self.administrator.key().as_ref()),
            Seed::from(core::slice::from_ref(&platform_state.bump)),
        ];
        let platform_signer = Signer::from(&platform_seeds);

        let user_uuid_str = self.user_uuid.to_string();
        let user_seeds = [
            Seed::from(USER_TOKEN),
            Seed::from(user_uuid_str.as_bytes()),
            Seed::from(self.platform_pda.key().as_ref()),
            Seed::from(core::slice::from_ref(&platform_state.bump)),
            Seed::from(core::slice::from_ref(&user_bump)),
        ];
        let user_signer = Signer::from(&user_seeds);

        CreateAccount {
            from: &self.administrator,
            to: &self.user_pda,
            lamports: min_lamports,
            space: User::INIT_SPACE as u64,
            owner: &ID,
        }
        .invoke_signed(&[platform_signer, user_signer])?;

        let user_meta = User::new(*self.platform_pda.key(), *self.user_uuid, user_bump);
        self.user_pda
            .try_borrow_mut_data()?
            .copy_from_slice(bytes_of(&user_meta));

        platform_state.total_users = platform_state
            .total_users
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        log!("platform pda created");
        Ok(())
    }
}

impl<'a> TryFrom<(&'a [AccountInfo], &'a [u8])> for CreateUser<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountInfo], &'a [u8])) -> Result<Self, Self::Error> {
        let (accounts, instruction_data) = value;

        let [administrator, platform_pda, user_pda, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if instruction_data.len() != size_of::<u128>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let user_uuid = try_from_bytes::<u128>(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self {
            administrator: administrator,
            platform_pda: platform_pda,
            user_pda: user_pda,
            user_uuid: user_uuid,
        })
    }
}
