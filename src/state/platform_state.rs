use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;

#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct PlatformState {
    pub discriminator: [u8; 8],
    pub authority: Pubkey, // this PDA should also be owned by authority
    pub fee_receiver: Pubkey,
    pub total_users: u64,
    pub total_mints: u64,
    pub mint_fee: u64,
    pub bump: u8,
    pub reserved: [u8; 128],
}

impl PlatformState {
    pub const DISCRIMINATOR: [u8; 8] = *b"platform";
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();

    pub fn new(authority: Pubkey, fee_receiver: Pubkey, bump: u8) -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR,
            authority: authority,
            fee_receiver: fee_receiver,
            total_users: 0,
            total_mints: 0,
            mint_fee: 0,
            bump: bump,
            reserved: [0; 128],
        }
    }
}
