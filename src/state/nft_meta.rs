use bytemuck::{Pod, Zeroable, try_from_bytes, try_from_bytes_mut};
use pinocchio::program_error::ProgramError;

#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct NftMeta {
    pub discriminator: [u8; 8],
    pub name: [u8; 256],
    pub collection: [u8; 64],
    pub uri: [u8; 256],
    pub description: [u8; 256],
}

impl NftMeta {
    pub const DISCRIMINATOR: [u8; 8] = *b"nftmeta\0";
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();

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
