# Vault Manager

A Solana program for managing token vaults. Users can create vaults, deposit tokens, and withdraw them.

## Prerequisites

- [Rust](https://rustup.rs/)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation)
- [Node.js](https://nodejs.org/) (v16 or later)
- [Yarn](https://yarnpkg.com/getting-started/install)

## Installation

1. Clone the repository:

```bash
$         git clone https://github.com/starke-labs/vault.git
$ cd vault
$ cp env.example .env
```

2. Set the `PROGRAM_AUTHORITY_SECRET_KEY` in the `.env` file to the secret key of the program authority and also update the `PROGRAM_AUTHORITY` in the `programs/vault_manager/src/lib.rs` file to the new program authority.

## Testing

To run the tests, use the following command:

```bash
$ anchor test
```

## Program Structure

- `programs/vault_manager/src/lib.rs`: Main program logic and instruction handlers
- `programs/vault_manager/src/vault.rs`: Vault account structure and methods
- `programs/vault_manager/src/vault_balance.rs`: VaultBalance account structure and methods
- `programs/vault_manager/src/event.rs`: Program event definitions
- `tests/`: TypeScript tests

## Development Notes and Todos

- Currently, each manager can only create one vault
- Currently, the vault is owned by the program itself, and the vault token account is owned by the vault - so the manager should not have authority over the vault token account (TODO: verify and confirm this!)
- Maximum number of tokens is currently set to 100
- Should we different whitelist for deposit tokens?
- TODO: Throw an error when trying to initialize whitelist twice
- TODO: Check if a different whitelist can be used for deposits and withdrawals
- TODO: Add vault.is_initialized() and whitelist.is_initialized() to throw the right error messages
