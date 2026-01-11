pub mod nft;
pub mod platform;
pub mod user;

use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;
use pinocchio_pubkey::declare_id;
use shank::ShankInstruction;

declare_id!("6jpuWYTM3ARc5CHrMBtR1c7gyjkMTsJoYT7PqqhMpRWh");

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
    UpdatePlatform {
        args: UpdatePlatformArgs,
    },

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
    CreateUser {
        user_uuid: u128,
    },
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
    #[account(5, name = "system_program")]
    MintNft {
        minNftArgs: MintNftArgs,
    },
    UpdateNFTMetadata,
    BurnNFT,

    TransferNFTInternal,
    WithdrawNFT,
    DepositNFT,
    /* TODO
    CreateAuction,
    PlaceBid,
    SettleAuction,
    */
}

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct UpdatePlatformArgs {
    pub mint_fee: u64,
    pub is_receiver_valid: u8,
    pub fee_receiver: Pubkey,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MintNftArgs {
    user_uuid: u128,
    asset_name: [u8; 256],
    desc: [u8; 256],
    uri: [u8; 256],
}
