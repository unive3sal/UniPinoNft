use alloc::string::ToString;
use bytemuck::{bytes_of, try_from_bytes, try_from_bytes_mut};
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

const USER_TOKEN: &[u8] = b"user_wallet";

pub struct CreateUser<'a> {
    pub authority: &'a AccountInfo,
    pub platform_pda: &'a AccountInfo,
    pub user_pda: &'a AccountInfo,
    pub user_uuid: u128,
}

impl<'a> CreateUser<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;
    pub const ACCOUNT_NUM: usize = 3;

    pub fn process(self) -> ProgramResult {
        if self.authority.is_signer() || self.platform_pda.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if self.platform_pda.is_owned_by(&ID) || self.platform_pda.lamports() == 0 {
            return Err(ProgramError::from(UniPinoNftErr::PlatformPdaUninit));
        }

        if self.user_pda.lamports() > 0 {
            return Err(ProgramError::from(UniPinoNftErr::UserPdaExisted));
        }

        let (user_pda, user_bump) = try_find_program_address(
            &[
                USER_TOKEN,
                self.user_uuid.to_string().as_bytes(),
                self.platform_pda.key().as_ref(),
            ],
            &ID,
        )
        .ok_or(UniPinoNftErr::PdaErr)
        .map_err(|e| ProgramError::from(e))?;

        if user_pda != self.user_pda.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }

        let min_lamports = Rent::get()?.minimum_balance(User::INIT_SPACE);
        log!("Init user PDA requires min balance: {}", min_lamports);

        let signer_seeds = [
            Seed::from(USER_TOKEN),
            Seed::from(self.authority.key().as_ref()),
            Seed::from(self.platform_pda.key().as_ref()),
            Seed::from(core::slice::from_ref(&user_bump)),
        ];

        let signer = Signer::from(&signer_seeds);
        CreateAccount {
            from: &self.authority,
            to: &self.user_pda,
            lamports: min_lamports,
            space: User::INIT_SPACE as u64,
            owner: &ID,
        }
        .invoke_signed(&[signer])?;

        let user_meta = User::new(*self.platform_pda.key(), self.user_uuid, user_bump);
        self.user_pda
            .try_borrow_mut_data()?
            .copy_from_slice(bytes_of(&user_meta));

        let mut platform_data_bytes = self
            .platform_pda
            .try_borrow_mut_data()
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let platform_state = try_from_bytes_mut::<Platform>(platform_data_bytes.as_mut())
            .map_err(|_| ProgramError::AccountBorrowFailed)?;
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
        if accounts.len() < Self::ACCOUNT_NUM {
            log!(
                "[Error] input accounts len: {}, require: {}",
                accounts.len(),
                Self::ACCOUNT_NUM
            );
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        if instruction_data.len() != size_of::<u128>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let user_uuid = try_from_bytes::<u128>(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        Ok(Self {
            authority: &accounts[0],
            platform_pda: &accounts[1],
            user_wallet_pda: &accounts[2],
            user_uuid: *user_uuid,
        })
    }
}
