# AGENTS.md - AI Coding Agent Guidelines

This document provides guidelines for AI coding agents working in this repository.

## Project Overview

- **Language:** Rust (Edition 2024, nightly features)
- **Framework:** Pinocchio (high-performance Solana program framework)
- **Domain:** Solana blockchain NFT program (native on-chain)
- **Program ID:** `6jpuWYTM3ARc5CHrMBtR1c7gyjkMTsJoYT7PqqhMpRWh`

## Build/Lint/Test Commands

```bash
# Build for Solana BPF deployment (primary)
cargo build-sbf

# Standard Rust build / check
cargo build
cargo check

# Run all tests
cargo test

# Run a single test by name
cargo test test_name

# Run exact test match
cargo test test_name -- --exact

# Run tests with output visible
cargo test -- --nocapture

# Format and lint
cargo fmt
cargo clippy
cargo clippy -- -D warnings
```

## Project Structure

```
src/
├── lib.rs                 # Program entrypoint and instruction routing
├── error.rs               # Custom error types (UniPinoNftErr)
├── instructions/
│   ├── mod.rs             # Instruction enum, argument types
│   ├── platform.rs        # Platform init/update handlers
│   ├── user.rs            # User creation handler
│   └── nft.rs             # NFT minting and metadata handlers
└── state/
    ├── platform.rs        # Platform PDA structure
    ├── user.rs            # User PDA structure
    └── nft_meta.rs        # NFT metadata structure
```

## Code Style Guidelines

### Import Style

Organize imports: 1) External crates, 2) `crate::`, 3) `super::`

```rust
use bytemuck::{bytes_of, try_from_bytes};
use pinocchio::{ProgramResult, account_info::AccountInfo, program_error::ProgramError};

use crate::error::UniPinoNftErr;
use crate::state::platform::Platform;

use super::*;
```

### Naming Conventions

- **Structs:** PascalCase (`Platform`, `MintNft`)
- **Functions:** snake_case (`process`, `try_from`)
- **Constants:** SCREAMING_SNAKE_CASE (`DISCRIMINATOR`, `INIT_SPACE`)
- **Modules/Files:** snake_case (`platform.rs`, `nft_meta.rs`)

### Error Handling

```rust
#[derive(Clone, Debug, PartialEq, Eq, Error, FromPrimitive)]
pub enum UniPinoNftErr {
    #[error("Fail to find a valid PDA")]
    PdaErr,
}

// Use ? with explicit error mapping
let (pda, bump) = try_find_program_address(&seeds, &ID)
    .ok_or(UniPinoNftErr::PdaErr)?;

// Always use checked arithmetic
platform.total_mints = platform.total_mints
    .checked_add(1)
    .ok_or(ProgramError::ArithmeticOverflow)?;
```

### State Struct Pattern

```rust
#[repr(C, packed)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Platform {
    pub discriminator: [u8; 8],
    pub field: u64,
    pub reserved: [u8; 128],  // For future upgrades
}

impl Platform {
    pub const DISCRIMINATOR: [u8; 8] = *b"platform";
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();
}
```

### Instruction Handler Pattern

```rust
pub struct InitPlatform<'a> {
    administrator: &'a AccountInfo,
    platform_pda: &'a AccountInfo,
}

impl<'a> InitPlatform<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(self) -> ProgramResult {
        if !self.administrator.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        // 1. Derive and validate PDA
        // 2. Create/modify accounts
        // 3. Update state
        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitPlatform<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [administrator, platform_pda, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        Ok(Self { administrator, platform_pda })
    }
}
```

### Key Libraries

| Library | Usage |
|---------|-------|
| pinocchio | Core Solana program framework |
| pinocchio-system | System program CPI |
| pinocchio-token-2022 | Token-2022 integration |
| bytemuck | Zero-copy serialization |
| shank | IDL generation |
| litesvm | Local VM testing (dev) |

### Special Rust Features

- `#![no_std]` - No standard library (required for Solana BPF)
- `extern crate alloc` - Heap allocation in no_std
- `#[repr(C, packed)]` - C-compatible packed structs

## Testing Guidelines

- Use LiteSVM for integration tests
- Test files go in `tests/` directory or `#[cfg(test)]` modules
- Name tests descriptively: `test_init_platform_success`

## Common Pitfalls

1. **Forgetting packed repr** - State structs must be `#[repr(C, packed)]`
2. **Missing discriminator check** - Always validate discriminator on deserialization
3. **Unchecked arithmetic** - Use `checked_add`, `checked_sub`, etc.
4. **Missing signer validation** - Check `is_signer()` for authority accounts
5. **PDA derivation mismatch** - Ensure seeds match between creation and lookup
