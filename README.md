# Starke Finance Vaults

A secure and flexible Solana program for trustless fund management with DeFi integrations.

## Overview

Starke Finance Vaults is a Solana program designed to bridge the gap between investors and fund managers by enabling the creation and management of token vaults (equivalent to traditional investment funds) in a trustless manner. The program allows fund managers to create and manage vaults while providing investors with secure, transparent, and verifiable investment opportunities through various DeFi integrations.

## Key Features

- **Vault Management**: Create and manage vaults with customizable parameters including entry/exit fees and token whitelisting
- **Token Operations**: Secure deposits and withdrawals with proportional vToken minting/burning based on NAV calculations
  - Supports both SPL Token and Token-2022 for vault assets. (Note: vTokens currently only support standard SPL Token mints.)
- **Price Integration**: Pyth price feed integration for accurate NAV calculations and token valuation
- **Security**: Program-controlled authority, comprehensive access controls, and time-delayed updates
- **Event System**: Detailed event emission for all major operations (vault creation, deposits, withdrawals, fee updates)
- **Swap Integration**: Built-in support for token swaps through Jupiter
- **Fee System**: Configurable entry and exit fees with a 30-day delay for updates to protect users (yet to be implemented, not for v0)
- **Transfer Controls**: Configurable vToken transfer restrictions through SPL Token-2022 transfer hook program

## Architecture

The program is structured into several key components:

- **Instructions**: Core program instructions for vault operations
- **State**: Account structures for vaults and whitelists
- **Controllers**: Business logic for token management and pricing
- **Constants**: Program-wide configuration values
- **Transfer Hook**: SPL Token-2022 transfer hook program for vToken transfer control

## Transfer Controls

The vaults program includes a transfer hook program that enables managers to control vToken transfers:

- **Configurable Transfers**: Managers can enable/disable vToken transfers for their vault
- **SPL Token-2022**: Utilizes the SPL Token-2022 program's transfer hook feature
- **Security**: Transfer restrictions are enforced at the protocol level
- **Flexibility**: Transfer settings can be updated by vault managers as needed

## Security

Security is a top priority. The program includes:

- Comprehensive access controls
- Time-delayed fee updates
- Token whitelisting
- Program authority controls
- Regular security audits

For reporting vulnerabilities, please see our [Security Policy](SECURITY.md).

For audit reports, please see the [audits](./audits) directory.

## Prerequisites

- [Rust](https://rustup.rs/)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation)
- [Node.js](https://nodejs.org/) (v16 or later)
- [Yarn](https://yarnpkg.com/getting-started/install)

## Installation

1. Clone the repository:

```bash
git clone https://github.com/starke-labs/vaults.git
cd vaults
```

2. Install dependencies:

```bash
yarn install
```

3. Build the program:

```bash
anchor build
```

4. Set up deployment keys:

```bash
mkdir deploy
cp <path-to-deploy-authority-keypair> deploy/authority.json
cp <path-to-vaults-keypair> target/deploy/vaults-keypair.json
```

## Testing

> NOTE: Tests are being refactored

Run the test suite:

```bash
anchor test
```

The tests cover:

- Whitelist management
- Vault creation and configuration
- Token deposits and withdrawals
- Fee updates
- Security controls
- Event emission

## Development Status

Current limitations and upcoming features:

- Currently, each manager can only create one vault
- Maximum number of tokens is currently set to 75
- TODO: Use `#[derive(InitSpace)]` for accounts instead of manually allocating space
- WIP: Refactor tests (use mainnet / devnet)
- TODO: Update sdk to new programs

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a new Pull Request

## License

AGPL License

See [LICENSE.md](LICENSE.md) for more details.

## Contact

- Discord: [Join our server](https://discord.gg/Kwvx8hcZBx)
- Email: contact@starke.finance
- Website: https://starke.finance

## Acknowledgments

- [Anchor Framework](https://www.anchor-lang.com/)
- [Pyth Network](https://pyth.network/)
- [Jupiter](https://jup.ag/)
