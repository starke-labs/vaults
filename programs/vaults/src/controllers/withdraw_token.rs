use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::*;

use super::{create_associated_token_account, transfer_token_with_signer};
use crate::constants::PRECISION;
use crate::state::{TokenWhitelist, Vault, VaultError, WhitelistError};

/// Withdraw all tokens from the vault token account to a user token account based on the withdrawal ratio
pub fn withdraw_all_tokens<'info>(
    remaining_accounts: &'info [AccountInfo<'info>],
    user: &Signer<'info>,
    vault: &Account<'info, Vault>,
    vtoken_mint: &Account<'info, Mint>,
    vtoken_amount: u64,
    whitelist: &Account<'info, TokenWhitelist>,
    signer_seeds: &[&[&[u8]]],
    token_program: &Program<'info, Token>,
    associated_token_program: &Program<'info, AssociatedToken>,
    system_program: &Program<'info, System>,
) -> Result<()> {
    msg!("Starting withdrawal of all tokens");
    msg!("Vtoken amount to withdraw: {}", vtoken_amount);

    // Calculate withdrawal ratio
    let total_supply = vtoken_mint.supply;
    msg!("Total vtoken supply: {}", total_supply);
    let withdrawal_ratio = calculate_withdrawal_ratio(vtoken_amount, total_supply)?;
    msg!("Withdrawal ratio calculated: {}", withdrawal_ratio);

    // Parse withdrawal accounts
    msg!("Parsing withdrawal accounts");
    let withdrawal_accounts = parse_withdrawal_accounts(
        remaining_accounts,
        user,
        whitelist,
        &vault.key(),
        token_program,
        system_program,
        associated_token_program,
    )?;
    msg!(
        "Found {} token accounts to process",
        withdrawal_accounts.len()
    );

    for (i, withdrawal_account) in withdrawal_accounts.iter().enumerate() {
        msg!(
            "Processing withdrawal {} of {} for mint: {}",
            i + 1,
            withdrawal_accounts.len(),
            withdrawal_account.mint.key()
        );
        withdraw_token(
            &withdrawal_account.mint,
            &withdrawal_account.vault_token_account,
            &withdrawal_account.user_token_account,
            withdrawal_ratio,
            vault,
            signer_seeds,
            token_program,
        )?;
        msg!("Successfully processed withdrawal {}", i + 1);
    }

    msg!("All token withdrawals completed successfully");
    Ok(())
}

/// Calculate the withdrawal ratio based on the amount of vtokens being withdrawn
/// and the total supply of vtokens in NAV decimals
fn calculate_withdrawal_ratio(amount: u64, total_supply: u64) -> Result<u64> {
    msg!(
        "Calculating withdrawal ratio - Amount: {}, Total Supply: {}",
        amount,
        total_supply
    );

    // Convert to u128 for intermediate calculations to prevent overflow
    let amount = amount as u128;
    let total_supply = total_supply as u128;
    let precision = PRECISION as u128;

    // Calculate (amount * PRECISION) / total_supply using u128
    let ratio = amount
        .checked_mul(precision)
        .ok_or(error!(VaultError::NumericOverflow))?
        .checked_div(total_supply)
        .ok_or(error!(VaultError::NumericOverflow))?;

    // Convert back to u64 after calculations
    let ratio = ratio as u64;
    msg!("Calculated withdrawal ratio: {}", ratio);
    Ok(ratio)
}

struct WithdrawalAccounts<'info> {
    pub mint: Account<'info, Mint>,
    pub vault_token_account: Account<'info, TokenAccount>,
    pub user_token_account: Account<'info, TokenAccount>,
}

/// Parse withdrawal accounts from remaining accounts
fn parse_withdrawal_accounts<'info>(
    remaining_accounts: &'info [AccountInfo<'info>],
    user: &Signer<'info>,
    whitelist: &Account<'info, TokenWhitelist>,
    vault_key: &Pubkey,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    associated_token_program: &Program<'info, AssociatedToken>,
) -> Result<Vec<WithdrawalAccounts<'info>>> {
    let mut withdrawal_accounts = Vec::new();

    for chunk in remaining_accounts.chunks(3) {
        // Each chunk must contain 3 accounts in this order:
        // 1. Token mint
        // 2. Vault token account
        // 3. User token account
        let mint = Account::<'info, Mint>::try_from(&chunk[0])?;
        let vault_token_account: Account<'info, TokenAccount> = Account::try_from(&chunk[1])?;

        msg!("Trying to parse user token account");
        if chunk[2].data_is_empty() {
            // User token account doesn't exist
            create_associated_token_account(
                user,
                &mint,
                &chunk[2],
                token_program,
                system_program,
                associated_token_program,
            )?;
        }

        let user_token_account: Account<'info, TokenAccount> = Account::try_from(&chunk[2])?;
        msg!("Successfully parsed user token account");

        msg!("Checking if mint and token account match");
        require!(
            whitelist.is_whitelisted(&mint.key()),
            WhitelistError::TokenNotWhitelisted
        );

        msg!("Checking if mint and vault token account match");
        require!(
            mint.key() == vault_token_account.mint,
            VaultError::MintAndTokenAccountMismatch
        );
        msg!("Checking if vault key and vault token account match");
        require!(
            *vault_key == vault_token_account.owner,
            VaultError::VaultAndTokenAccountMismatch
        );

        // TODO: What happens if the user token account doesn't exist yet?
        msg!("Checking if mint and user token account match");
        require!(
            mint.key() == user_token_account.mint,
            VaultError::MintAndTokenAccountMismatch
        );
        msg!("Checking if user and user token account match");
        require!(
            user.key() == user_token_account.owner,
            VaultError::UserAndTokenAccountMismatch
        );

        withdrawal_accounts.push(WithdrawalAccounts {
            mint,
            vault_token_account,
            user_token_account,
        });
    }

    Ok(withdrawal_accounts)
}

/// Withdraw a single token from the vault token account to a user token account
fn withdraw_token<'info>(
    mint: &Account<'info, Mint>,
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    withdrawal_ratio: u64,
    vault: &Account<'info, Vault>,
    signer_seeds: &[&[&[u8]]],
    token_program: &Program<'info, Token>,
) -> Result<()> {
    // TODO: Throw error if to account (user token account) key doesn't match
    //       the one associated with the user and the mint
    // require!(...)
    msg!("Processing withdrawal for mint: {}", mint.key());
    msg!("From account: {}", from.key());
    msg!("To account: {}", to.key());

    // Calculate the amount of tokens to withdraw based on the withdrawal ratio
    let token_balance = from.amount;
    msg!("Vault token account balance: {}", token_balance);
    let amount = calculate_token_withdrawal_amount(token_balance, withdrawal_ratio)?;
    msg!("Amount to withdraw: {}", amount);

    // Transfer tokens from vault to withdrawer
    msg!("Transferring tokens from vault to user");
    transfer_token_with_signer(
        from,
        to,
        amount,
        &vault.to_account_info(),
        signer_seeds,
        token_program,
    )?;
    msg!("Token transfer completed successfully");

    Ok(())
}

/// Calculate the amount of tokens to withdraw for a specific
/// token balance and withdrawal ratio in NAV decimals
fn calculate_token_withdrawal_amount(token_balance: u64, withdrawal_ratio: u64) -> Result<u64> {
    msg!(
        "Calculating token withdrawal amount - Balance: {}, Ratio: {}",
        token_balance,
        withdrawal_ratio
    );
    let amount = token_balance
        .checked_mul(withdrawal_ratio)
        .ok_or(error!(VaultError::NumericOverflow))?
        .checked_div(PRECISION)
        .ok_or(error!(VaultError::NumericOverflow))?;
    msg!("Calculated withdrawal amount: {}", amount);
    Ok(amount)
}
