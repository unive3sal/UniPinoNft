use bytemuck::{Pod, Zeroable};

#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct User {
    pub discriminator: u8,
    pub user_uuid: u128,
    pub authority: [u8; 32],
    pub nft_count: u32,
    pub collection_count: u32,
    pub bump: u8,
    pub reserved: [u8; 64],
}

impl User {
    pub const DISCRIMINATOR: u8 = 0x23;
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();

    pub fn new(platform_pda: [u8; 32], user_uuid: u128, user_bump: u8) -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR,
            user_uuid,
            authority: platform_pda,
            nft_count: 0,
            collection_count: 0,
            bump: user_bump,
            reserved: [0; 64],
        }
    }

    crate::state::impl_state_accessors!(Self);
}
