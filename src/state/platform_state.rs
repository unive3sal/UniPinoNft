use pinocchio::pubkey::Pubkey;

#[repr(C)]
pub struct PlatformState {
    pub discriminator: [u8; 8],
    pub authority: Pubkey,  // this PDA should also be owned by authority
    pub total_users: u64,
    pub total_mints: u64,
    pub mint_fee: u64,
    pub bump: u8,
    pub reserved: [u8; 128],
}
