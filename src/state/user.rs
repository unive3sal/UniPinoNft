use bytemuck::{Pod, Zeroable};
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
            user_uuid: user_uuid,
            owner: platform_pda,
            nft_count: 0,
            collection_count: 0,
            bump: user_bump,
            reserved: [0; 64],
        }
    }
}
