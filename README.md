# Starke Finance Vaults

A secure and flexible Solana program for managing token vaults with advanced features for DeFi applications.

## Overview

Starke Finance Vaults is a Solana program that enables:

- Creation and management of token vaults
- Secure token deposits and withdrawals
- Configurable entry/exit fees
- Token whitelisting with price feed integration
- Integration with Jupiter for token swaps
- Advanced security features and access controls

## Key Features

- **Vault Management**: Create and manage vaults with customizable parameters
- **Token Whitelisting**: Only approved tokens can be used with built-in Pyth price feed integration
- **Fee Structure**: Configurable entry and exit fees with time-delayed updates
- **Security**: Program-controlled authority and comprehensive access controls
- **Event Emission**: Detailed events for all major operations
- **Jupiter Integration**: Built-in support for token swaps (coming soon)

## Architecture

The program is structured into several key components:

- **Instructions**: Core program instructions for vault operations
- **State**: Account structures for vaults and whitelists
- **Controllers**: Business logic for token management and pricing
- **Constants**: Program-wide configuration values

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

## Security

Security is a top priority. The program includes:

- Comprehensive access controls
- Time-delayed fee updates
- Token whitelisting
- Program authority controls
- Regular security audits

For reporting vulnerabilities, please see our [Security Policy](SECURITY.md).

## Development Status

Current limitations and upcoming features:

- Currently, each manager can only create one vault
- Currently, the vault is owned by the program itself, and the vault token account is owned by the vault - so the manager should not have authority over the vault token account (TODO: verify and confirm this!)
- Maximum number of tokens is currently set to 100
- TODO: Add swap logic here from: https://github.com/starke-labs/jup-swap-integration

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
- Email: contact@starkevalidator.com
- Website: https://starke.finance

## Acknowledgments

- [Anchor Framework](https://www.anchor-lang.com/)
- [Pyth Network](https://pyth.network/)
- [Jupiter](https://jup.ag/)