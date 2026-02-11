pub mod nft;
pub mod platform;
pub mod user;

use bytemuck::{try_from_bytes, Pod, Zeroable};
use pinocchio::error::ProgramError;

pub fn parse_instruction_data<T: Pod>(instruction_data: &[u8]) -> Result<&T, ProgramError> {
    if instruction_data.len() != core::mem::size_of::<T>() {
        return Err(ProgramError::InvalidInstructionData);
    }

    try_from_bytes::<T>(instruction_data).map_err(|_| ProgramError::InvalidInstructionData)
}

#[cfg(test)]
mod tests {
    use bytemuck::{bytes_of, Pod, Zeroable};
    use pinocchio::error::ProgramError;

    use super::parse_instruction_data;

    #[repr(C)]
    #[derive(Clone, Copy, Pod, Zeroable)]
    struct DummyArgs {
        value: u64,
    }

    #[test]
    fn parse_instruction_data_rejects_bad_size() {
        let payload = [1_u8; 7];
        let result = parse_instruction_data::<DummyArgs>(&payload);

        assert!(matches!(result, Err(ProgramError::InvalidInstructionData)));
    }

    #[test]
    fn parse_instruction_data_returns_typed_view() {
        let args = DummyArgs { value: 42 };
        let payload = bytes_of(&args);

        let parsed = parse_instruction_data::<DummyArgs>(payload).unwrap();

        assert_eq!(parsed.value, 42);
    }
}

/*
use shank::ShankInstruction;
#[allow(dead_code)]
#[derive(ShankInstruction)]
pub enum UniPinoNftInstruction {
    #[account(
        0,
        signer,
        writable,
        name = "authority account",
        desc = "init account, and it is responsible for paying gas and NFT rent"
    )]
    #[account(
        1,
        name = "platform PDA",
        desc = "account for on-chain platform management"
    )]
    #[account(2, name = "system_program")]
    InitPlatform,

    #[account(
        0,
        signer,
        writable,
        name = "authority account",
        desc = "update platform"
    )]
    #[account(
        1,
        writable,
        name = "platform PDA",
        desc = "account for on-chain platform management"
    )]
    #[account(2, name = "system_program")]
    UpdatePlatform { args: UpdatePlatformArgs },

    #[account(
        0,
        signer,
        writable,
        name = "authority account",
        desc = "init account, and it is responsible for paying gas and NFT rent"
    )]
    #[account(
        1,
        writable,
        name = "platform PDA",
        desc = "account for on-chain platform management"
    )]
    #[account(2, writable, name = "user wallet PDA")]
    #[account(3, name = "system_program")]
    CreateUser { user_uuid: u128 },
    /* TODO
    ActivateUserWallet,
    DeactivateUserWallet,
    */
    #[account(
        0,
        signer,
        writable,
        name = "authority account",
        desc = "init account, and it is responsible for paying gas and NFT rent"
    )]
    #[account(
        1,
        writable,
        name = "platform PDA",
        desc = "account for on-chain platform management"
    )]
    #[account(2, writable, name = "user PDA")]
    #[account(3, writable, name = "mint PDA")]
    #[account(4, writable, name = "metadata PDA")]
    #[account(
        5,
        writable,
        name = "fee_receiver",
        desc = "account to receive mint fees"
    )]
    #[account(6, name = "system_program")]
    MintNft { mint_nft_args: MintNftArgs },

    #[account(
        0,
        signer,
        writable,
        name = "authority account",
        desc = "init account, and it is responsible for paying gas and NFT rent"
    )]
    #[account(
        1,
        writable,
        name = "platform PDA",
        desc = "account for on-chain platform management"
    )]
    #[account(2, writable, name = "user PDA")]
    #[account(3, writable, name = "mint PDA")]
    #[account(4, writable, name = "metadata PDA")]
    #[account(5, name = "system_program")]
    UpdateNFTMetadata { nft_meta: NftMeta },

    #[account(
        0,
        signer,
        writable,
        name = "authority account",
        desc = "init account, and it is responsible for paying gas and NFT rent"
    )]
    #[account(
        1,
        writable,
        name = "platform PDA",
        desc = "account for on-chain platform management"
    )]
    #[account(2, writable, name = "user PDA")]
    #[account(3, writable, name = "mint PDA")]
    #[account(4, writable, name = "metadata PDA")]
    #[account(5, name = "system_program")]
    BurnNFT,
    /* TODO
    TransferNFTInternal,
    WithdrawNFT,
    DepositNFT,
    CreateAuction,
    PlaceBid,
    SettleAuction,
    */
}
*/

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct UpdatePlatformArgs {
    pub mint_fee: u64,
    pub is_receiver_valid: u8,
    pub fee_receiver: [u8; 32],
}

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MintNftArgs {
    pub user_uuid: u128,
    pub name: [u8; 128],
    pub uri: [u8; 256],
    pub update_authority_type: u8,
    pub update_authority: [u8; 32],
}

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct NftMetaArgs {
    pub update_authority_type: u8,
    pub update_authority: [u8; 32],
    pub name: [u8; 128],
    pub uri: [u8; 256],
}
