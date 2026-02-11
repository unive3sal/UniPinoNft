use bytemuck::{Pod, Zeroable};

/// Fixed-size bytemuck mirror of Metaplex Core `AssetV1`.
/// `name` and `uri` are zero-padded UTF-8; `update_authority_type` encodes
/// the `UpdateAuthority` enum tag (0 = None, 1 = Address, 2 = Collection).
#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct NftMeta {
    pub discriminator: u8,
    /// 0 = None, 1 = Address, 2 = Collection (see `update_authority_type` mod).
    pub update_authority_type: u8,
    pub update_authority: [u8; 32],
    pub name: [u8; 128],
    pub uri: [u8; 256],
    pub seq: u64,
    pub reserved: [u8; 64],
}

pub mod update_authority_type {
    pub const NONE: u8 = 0;
    pub const ADDRESS: u8 = 1;
    pub const COLLECTION: u8 = 2;
}

pub mod asset_key {
    pub const UNINITIALIZED: u8 = 0;
    pub const ASSET_V1: u8 = 1;
    pub const HASHED_ASSET_V1: u8 = 2;
    pub const PLUGIN_HEADER_V1: u8 = 3;
    pub const PLUGIN_REGISTRY_V1: u8 = 4;
    pub const COLLECTION_V1: u8 = 5;
}

impl NftMeta {
    pub const DISCRIMINATOR: u8 = asset_key::ASSET_V1;
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();
}
