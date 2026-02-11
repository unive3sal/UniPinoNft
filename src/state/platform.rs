use bytemuck::{Pod, Zeroable};

#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Platform {
    pub discriminator: u8,
    pub authority: [u8; 32],
    pub fee_receiver: [u8; 32],
    pub total_users: u64,
    pub total_mints: u64,
    pub mint_fee: u64,
    pub bump: u8,
    pub reserved: [u8; 128],
}

impl Platform {
    pub const DISCRIMINATOR: u8 = 0x13;
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();

    pub fn new(authority: [u8; 32], bump: u8) -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR,
            authority,
            fee_receiver: authority,
            total_users: 0,
            total_mints: 0,
            mint_fee: 0,
            bump,
            reserved: [0; 128],
        }
    }

    crate::state::impl_state_accessors!(Self);
}
