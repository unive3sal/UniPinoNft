pub mod platform;
pub mod user_wallet;

use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;
use pinocchio_pubkey::declare_id;
use shank::ShankInstruction;

declare_id!("6jpuWYTM3ARc5CHrMBtR1c7gyjkMTsJoYT7PqqhMpRWh");

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
        signer,
        writable,
        name = "platform PDA",
        desc = "account for on-chain platform management"
    )]
    #[account(2, writable, name = "user wallet PDA")]
    #[account(3, name = "system_program")]
    CreateUserWallet {
        user_uuid: u128,
    },

    ActivateUserWallet,
    DeactivateUserWallet,

    MintNFT,
    UpdateNFTMetadata,
    BurnNFT,

    TransferNFTInternal,
    WithdrawNFT,
    DepositNFT,

    CreateAuction,
    PlaceBid,
    SettleAuction,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct UpdatePlatformArgs {
    pub is_receiver_valid: u8,
    pub fee_receiver: Pubkey,
    pub mint_fee: u64,
}
