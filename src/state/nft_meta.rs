use bytemuck::{Pod, Zeroable};

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
}
