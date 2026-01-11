use bytemuck::{Pod, Zeroable};
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
            bump: bump,
            reserved: [0; 128],
        }
    }
}
