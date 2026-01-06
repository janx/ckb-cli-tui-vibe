# AGENTS.md - Coding Agent Guidelines for ckb-cli

## Project Overview

CKB command-line interface tool for Nervos CKB blockchain. Rust edition 2021, toolchain 1.85.0.

## Build Commands

```bash
make prod                    # Build release binary (cargo build --locked --release)
cargo build                  # Build debug binary
```

## Lint Commands

```bash
make fmt                     # Check formatting (cargo fmt --all -- --check)
make clippy                  # Run clippy lints
```

Clippy config (in Makefile): `-D warnings -D clippy::clone_on_ref_ptr -D clippy::enum_glob_use -A clippy::mutable_key_type -A clippy::upper_case_acronyms`

## Test Commands

### Unit Tests
```bash
make test                              # Run all unit tests
cargo test utils::tx_helper            # Run tests for a module
cargo test test_check_lock_script      # Run a single test function
```

### Integration Tests
```bash
make integration                       # Run all (downloads CKB node)
make integration-spec SPEC=wallet      # Run tests matching "wallet"
SPEC_FILTER=deploy make integration    # Alternative filter method
```

Integration specs in `test/src/spec/`: `AccountKeystorePerm`, `WalletTransfer`, `DaoPrepareOne`, `DeployDepGroupWithTypeId`, `SudtIssueToCheque`, etc.

## Code Style

### Import Organization
Group in order: 1) std, 2) external crates, 3) internal modules
```rust
use std::collections::HashMap;

use ckb_sdk::{Address, NetworkType};
use clap::{App, Arg, ArgMatches};

use crate::utils::config::GlobalConfig;
```

### Naming Conventions
| Element | Convention | Example |
|---------|------------|---------|
| Functions/Variables | snake_case | `get_genesis_info`, `ckb_cli_dir` |
| Structs/Enums/Traits | CamelCase | `WalletSubCommand`, `ArgParser` |
| Constants | SCREAMING_SNAKE_CASE | `SIGHASH_TYPE_HASH` |

Short names acceptable locally: `m` for ArgMatches, `tx` for transaction.

### Error Handling
Use `Result<T, String>` for CLI functions with `.map_err(|e| e.to_string())`:
```rust
pub fn process(&mut self, matches: &ArgMatches) -> Result<Output, String> {
    let value = self.client.get_block(hash).map_err(|e| e.to_string())?;
    Ok(Output::new_output(value))
}
```

For library code (`ckb-signer`), use `thiserror`:
```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("Account not found: {0:x}")]
    AccountNotFound(H160),
}
```

### Subcommand Pattern
```rust
pub trait CliSubCommand {
    fn process(&mut self, matches: &ArgMatches, debug: bool) -> Result<Output, String>;
}

pub struct MySubCommand<'a> {
    rpc_client: &'a mut HttpRpcClient,
}

impl<'a> MySubCommand<'a> {
    pub fn new(...) -> Self { ... }
    pub fn subcommand(name: &'static str) -> App<'static> { ... }
}
```

### Unit Test Pattern
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_name() {
        let result = function_under_test(input);
        assert_eq!(result, expected);
    }
}
```

## Project Structure
```
src/
├── main.rs              # Entry point
├── interactive.rs       # REPL mode
├── plugin/              # Plugin management
├── subcommands/         # CLI commands (account.rs, wallet.rs, dao/, etc.)
└── utils/               # Shared utilities (arg_parser.rs, rpc/, tx_helper.rs)

ckb-signer/              # Keystore library
plugin-protocol/         # Plugin communication
test/                    # Integration tests (separate workspace)
```

## Security & Auditing
```bash
make security-audit      # Vulnerability audit
make check-crates        # Banned/duplicate crates
make check-licenses      # License compliance
```

Allowed licenses (deny.toml): MIT, Apache-2.0, BSD-2/3-Clause, ISC, CC0-1.0, MPL-2.0, Unicode-*.

## Common Gotchas

1. **Cargo.lock sync**: Integration tests need `cp -f Cargo.lock test/Cargo.lock`
2. **Clap version**: Using beta `clap = "=3.0.0-beta.1"` - API differs from stable
3. **Unix-only**: TUI module only compiles on Unix (`#[cfg(unix)]`)
4. **Env vars**: `API_URL` for RPC, `CKB_CLI_HOME` for config directory

## CI Requirements

All PRs must pass: `make fmt`, `make clippy`, `make test`, `make integration`, `cargo deny check`. No unintentional `Cargo.lock` changes.
