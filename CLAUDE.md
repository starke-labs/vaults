# Starke Vaults — Claude Code instructions

## Context
- Solana programs for Starke tokenized funds: `programs/vaults`, `programs/transfer_hook`, `programs/jupiter` (CPI interface only).
- LIVE on mainnet. Correctness beats speed, always.
- This repository is PUBLIC. Everything committed, pushed, or written in PR and issue text is visible to anyone.

## Stack
- Anchor 0.31.1 with the Solana toolchain pinned in Anchor.toml (migration to Anchor 1.x planned)
- Tests: LiteSVM / Mollusk (unit), Surfpool (mainnet-fork integration).
- Oracles: Pyth (pull) and Pyth Pro. Swaps: Jupiter via CPI.

## Commands
- Build: `anchor build`
- Unit tests: `cargo test`
- Lint: `cargo fmt --check && cargo clippy -- -D warnings`

## Branches and PRs
- Work on feature branches named after the Linear issue; open PRs against `sandpit`.
- Never push to `sandpit`, `staging` or `main` directly; promotion between them is done by humans.

## Workflow (every task)
1. Plan mode first: accounts touched, new state, checks, tests. Wait for approval.
2. Tests first; a change under `programs/` without tests is incomplete.
3. Smallest change that passes; run lint and tests before asking for review.
4. Run the `security-reviewer` subagent on the diff.
5. Small PR: what changed, why, account-layout impact, IDL/event impact, test evidence.

## On-chain rules (never break)
- Never reorder, resize or remove fields in an existing account; append, or migrate.
- Every instruction: explicit signer, owner, PDA seed/bump checks and `has_one` constraints.
- Checked math only; no `unwrap()` or truncating `as` casts in program code.
- Validate every remaining account: owner, mint, expected set, no duplicates.
- Oracle prices: check staleness and confidence.
- No new `UncheckedAccount` without a `/// CHECK:` comment explaining why it is safe.
- Emit an event for every state change investors or indexers rely on; never change event field order or log strings without flagging it.
- Never run `anchor keys sync`. Program IDs are fixed per cluster; the Jupiter ID must stay `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`.

## Architecture direction
- A small core holds custody and share accounting and enforces invariants; configuration, pricing and strategy live in separate modules. New features go where that design places them, not into the existing program by default.

## Environments and secrets
- Deploy only to localnet/Surfpool for now. Never deploy, upgrade or sign anything on mainnet, staging program included; mainnet is done by humans.
- Never read, print, copy or create keypairs, tokens or seed phrases; never put RPC URLs with keys in code, logs or commits.

## Style
- One concept per file: `instructions/`, `state/`, `controllers/`, `constants.rs`, `errors.rs`.
- `#[derive(InitSpace)]` for new accounts; doc comments on every public instruction and account field.
- Clear code over compute-unit micro-optimizations unless a test shows a limit is hit.
