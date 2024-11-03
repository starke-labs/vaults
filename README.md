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
$ git clone https://github.com/starke-labs/vault.git
$ cd vault
```

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
- We can not let the manager have authority over the vault token account because it would allow the manager to steal funds from the vault
  - The owner of the vault token account should be the program itself?
