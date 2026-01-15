# UniPinoNft

A Solana NFT program built with the [Pinocchio](https://github.com/febo/pinocchio) framework for high-performance, low-level program development.

## Overview

UniPinoNft is a native Solana program that provides NFT minting and management capabilities using the Token-2022 program. It features a platform-based architecture with administrator controls, user wallet management via PDAs, and NFT metadata storage.

**Program ID:** `6jpuWYTM3ARc5CHrMBtR1c7gyjkMTsJoYT7PqqhMpRWh`

## Features

- **Platform Management** - Initialize and configure platform settings including mint fees and fee receivers
- **User Wallet PDAs** - Create and manage user accounts derived from platform PDAs
- **NFT Minting** - Mint NFTs using Token-2022 with on-chain metadata storage
- **Fee Configuration** - Configurable minting fees with designated fee receiver

## Architecture

### State Accounts

| Account | Description |
|---------|-------------|
| `Platform` | Stores platform configuration: administrator, fee receiver, total users/mints, mint fee |
| `User` | User wallet PDA containing UUID, owner reference, NFT count, collection count |
| `NftMeta` | NFT metadata including name, collection, URI, and description |

### Instructions

| Instruction | Discriminator | Description |
|-------------|---------------|-------------|
| `InitPlatform` | 0 | Initialize platform PDA with administrator |
| `UpdatePlatformConfig` | 1 | Update platform fee settings and receiver |
| `CreateUser` | 2 | Create a new user wallet PDA |
| `MintNft` | 3 | Mint a new NFT with metadata |

## Project Structure

```
src/
├── lib.rs                 # Program entrypoint and instruction routing
├── error.rs               # Custom error types
├── instructions/
│   ├── mod.rs             # Instruction enum and argument types
│   ├── platform.rs        # Platform init and update handlers
│   ├── user.rs            # User creation handler
│   └── nft.rs             # NFT minting handler
└── state/
    ├── platform.rs        # Platform account structure
    ├── user.rs            # User account structure
    └── nft_meta.rs        # NFT metadata structure
```

## Dependencies

- **pinocchio** - Core Solana program framework
- **pinocchio-system** - System program CPI helpers
- **pinocchio-token-2022** - Token-2022 program integration
- **bytemuck** - Zero-copy serialization
- **shank** - IDL generation

## Building

```bash
cargo build-sbf
```

## Testing

```bash
cargo test
```

Uses [LiteSVM](https://github.com/LiteSVM/litesvm) for local testing.

## PDA Seeds

### Platform PDA
```
seeds = ["administer", administrator_pubkey]
```

### User PDA
```
seeds = ["user_wallet", user_uuid_string, platform_pda, platform_bump]
```

### Mint PDA
```
seeds = [user_uuid_bytes, user_pda, TOKEN_2022_ID]
program = TOKEN_2022_ID
```

### Metadata PDA
```
seeds = ["metadata", mint_pda, TOKEN_2022_ID]
program = TOKEN_2022_ID
```

## License

GPL-3.0
