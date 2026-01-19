use bytemuck::{Pod, Zeroable, try_from_bytes, try_from_bytes_mut};
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::Pubkey;

#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Platform {
    pub discriminator: [u8; 8],
    pub administrator: Pubkey, // this PDA should also be owned by authority
    pub fee_receiver: Pubkey,
    pub total_users: u64,
    pub total_mints: u64,
    pub mint_fee: u64,
    pub bump: u8,
    pub reserved: [u8; 128],
}

impl Platform {
    pub const DISCRIMINATOR: [u8; 8] = *b"platform";
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();

    pub fn new(authority: Pubkey, bump: u8) -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR,
            administrator: authority,
            fee_receiver: authority,
            total_users: 0,
            total_mints: 0,
            mint_fee: 0,
            bump,
            reserved: [0; 128],
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
