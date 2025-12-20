use alloc::string::ToString;
use bytemuck::{try_from_bytes, try_from_bytes_mut};
use pinocchio::ProgramResult;
use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::try_find_program_address;
use pinocchio_log::log;

use crate::error::UniPinoNftErr;
use crate::state::platform_state::PlatformState;

use super::*;

const USER_TOKEN: &[u8] = b"user_wallet";

pub struct CreateUserWallet<'a> {
    pub authority: &'a AccountInfo,
    pub platform_pda: &'a AccountInfo,
    pub user_wallet_pda: &'a AccountInfo,
    pub user_uuid: u128,
}

impl<'a> CreateUserWallet<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;
    pub const ACCOUNT_NUM: usize = 3;

    pub fn process(self) -> ProgramResult {
        if self.authority.is_signer() || self.platform_pda.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if self.platform_pda.is_owned_by(&ID) || self.platform_pda.lamports() == 0 {
            return Err(ProgramError::from(UniPinoNftErr::PlatformPdaUninit));
        }

        let mut platform_data_bytes = self
            .platform_pda
            .try_borrow_mut_data()
            .map_err(|_| ProgramError::InvalidAccountData)?;
        let platform_state = try_from_bytes_mut::<PlatformState>(platform_data_bytes.as_mut())
            .map_err(|_| ProgramError::AccountBorrowFailed)?;

        if self.user_wallet_pda.lamports() > 0 {
            return Err(ProgramError::from(UniPinoNftErr::UserPdaExisted));
        }

        let (user_pda, user_bump) =
            try_find_program_address(&[USER_TOKEN, self.user_uuid.to_string().as_bytes()], &ID)
                .ok_or(UniPinoNftErr::PdaErr)
                .map_err(|e| ProgramError::from(e))?;

        if user_pda != self.user_wallet_pda.key().as_ref() {
            return Err(ProgramError::InvalidSeeds);
        }

        Ok(())
    }
}

impl<'a> TryFrom<(&'a [AccountInfo], &'a [u8])> for CreateUserWallet<'a> {
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
