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
- Currently, the vault is owned by the program itself, and the vault token account is owned by the vault - so the manager should not have authority over the vault token account (TODO: verify and confirm this!)

## Fund Management Strategy

### 1. Define Allocation Strategy

- Allocation Structure:
  - Allow fund managers to specify a portfolio of whitelisted assets along with their target allocation percentages.
  - Store this allocation data in the vault's state for easy access and updates.

- Asset Whitelisting:
  - Maintain a list of approved assets that can be included in the vault and can only be updated by the admin.

### 2. Rebalancing Mechanism

- Periodic Rebalancing:
  - Implement a rebalancing function that adjusts the vault's holdings to match the target allocations.
  - Rebalancing can be triggered by:
    - Significant market movements causing drift from target allocations.
    - Manager-initiated changes in allocation weights.
    - Accumulated deposits or withdrawals reaching a threshold.

- Rebalancing Strategies:
  - Threshold-Based Rebalancing: Rebalance when an asset's actual allocation deviates from its target by a predefined percentage.
  - Time-Based Rebalancing: Rebalance at regular intervals (e.g., weekly, monthly).
  - Comparative Analysis:
    - Cost Considerations:
      - Threshold-Based: May incur higher costs during volatile periods due to increased trading.
      - Time-Based: Provides predictable costs but may trade unnecessarily, leading to potential waste.
    - Risk Management:
      - Threshold-Based: Better at maintaining desired risk levels by promptly correcting significant drifts.
      - Time-Based: Risk levels may fluctuate between rebalancing dates, potentially diverging from the investor's risk tolerance.
    - Operational Complexity:
      - Threshold-Based: Requires sophisticated monitoring systems and readiness to act at any time.
      - Time-Based: Simpler to implement with scheduled activities and reduced need for constant oversight.
    - Tax Implications:
      - Threshold-Based: May result in more taxable events due to frequent rebalancing.
      - Time-Based: Potentially more tax-efficient with fewer trades.
    - Investor Communication:
      - Threshold-Based: Harder to predict and explain rebalancing actions to investors.
      - Time-Based: Easier to communicate and set expectations with investors.
    - Performance Impact:
      - Threshold-Based: May enhance performance by maintaining strategic allocations but can suffer in volatile markets due to costs.
      - Time-Based: May lag in performance if allocations drift significantly but benefits from reduced trading costs.

### 3. Handling Deposits

- Immediate Swapping vs Batching:
  - Deposit Queue:
    - Collect deposits in a queue and process them when a certain volume or time threshold is met.
  - User Options:
    - Allow users to choose between immediate processing (with higher fees) or batched processing.

- Allocation of Deposits:
  - Proportional Allocation:
    - When processing deposits, distribute the funds across the target assets based on current allocation percentages.
  - Handling Single Asset Deposits:
    - Accept deposits in the vault's base asset (e.g., USDC).
    - Swap the base asset into the target assets during allocation.

### 4. Handling Withdrawals

- Withdrawal Requests:
  - Processing Withdrawals:
    - Calculate the user's share of each asset in the vault.
    - Option to provide users with:
      - Proportional amounts of each asset.
      - Swapped amount in a single asset (e.g., USDC), requiring additional swaps.

- Liquidity Management:
  - Reserve Pool:
    - Maintain a liquidity buffer to handle immediate withdrawals without affecting the vault's overall strategy.
  - Batching Withdrawals:
    - Similar to deposits, batch withdrawal requests to minimize transaction costs.

- Withdrawal Fees and Penalties:
  - Early Withdrawal Penalties:
    - Implement fees for withdrawals made before a certain period to discourage frequent transactions.

### 5. Changing Allocation Weightages

- Updating Allocations:
  - Allow fund managers to update target allocations through authenticated transactions.

- Rebalancing Post-Allocation Change:
  - Efficient Rebalancing:
    - Calculate the minimal set of trades needed to achieve the new allocations.
    - Use netting to offset buy and sell orders.
  - Cost Management:
    - Consider the market impact and transaction fees when rebalancing.
    - Possibly spread rebalancing over time to minimize slippage.
