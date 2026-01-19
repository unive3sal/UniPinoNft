use bytemuck::{Pod, Zeroable, try_from_bytes, try_from_bytes_mut};
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::Pubkey;

#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct User {
    pub discriminator: [u8; 8],
    pub user_uuid: u128,
    pub owner: Pubkey, // this PDA is owned by platform PDA
    pub nft_count: u32,
    pub collection_count: u32,
    pub bump: u8,
    pub reserved: [u8; 64],
}

impl User {
    pub const DISCRIMINATOR: [u8; 8] = *b"usermeta";
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();

    pub fn new(platform_pda: Pubkey, user_uuid: u128, user_bump: u8) -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR,
            user_uuid,
            owner: platform_pda,
            nft_count: 0,
            collection_count: 0,
            bump: user_bump,
            reserved: [0; 64],
        }
    }

    /// Deserialize and validate discriminator for immutable access
    pub fn try_from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        let state = try_from_bytes::<Self>(data).map_err(|_| ProgramError::InvalidAccountData)?;
        if state.discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(state)
    }

    /// Deserialize and validate discriminator for mutable access
    pub fn try_from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        let state =
            try_from_bytes_mut::<Self>(data).map_err(|_| ProgramError::InvalidAccountData)?;
        if state.discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(state)
    }
}
