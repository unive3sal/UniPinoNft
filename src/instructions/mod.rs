pub mod platform_management;

use pinocchio_pubkey::declare_id;
use shank::ShankInstruction;

declare_id!("6jpuWYTM3ARc5CHrMBtR1c7gyjkMTsJoYT7PqqhMpRWh");

#[derive(ShankInstruction)]
pub enum UniPinoNftInstruction {
    #[account(0, signer, writable, name="authority account",
        desc="init account, and it is responsible for paying gas and NFT rent")]
    #[account(1, name="plateform PDA",
        desc="account for on-chain plateform management")]
    #[account(2, name="system_program")]
    InitPlatform,

    UpdatePlatform,

    #[account(0, signer, writable, name="authority account",
        desc="init account, and it is responsible for paying gas and NFT rent")]
    #[account(1, writable, name="user wallet PDA")]
    #[account(2, writable, name="plateform PDA",
        desc="account for on-chain plateform management")]
    #[account(3, name="system_program")]
    CreateUserWallet {user_uuid: u128},

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
