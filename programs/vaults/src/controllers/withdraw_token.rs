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
    vault_token_mint: &Account<'info, Mint>,
    vault_token_amount: u64,
    whitelist: &Account<'info, TokenWhitelist>,
    signer_seeds: &[&[&[u8]]],
    token_program: &Program<'info, Token>,
    associated_token_program: &Program<'info, AssociatedToken>,
    system_program: &Program<'info, System>,
) -> Result<()> {
    // Calculate withdrawal ratio
    let total_supply = vault_token_mint.supply;
    let withdrawal_ratio = calculate_withdrawal_ratio(vault_token_amount, total_supply)?;

    // Parse withdrawal accounts
    let withdrawal_accounts =
        parse_withdrawal_accounts(remaining_accounts, user, whitelist, vault.key())?;

    for withdrawal_account in withdrawal_accounts {
        withdraw_token(
            &withdrawal_account.mint,
            &withdrawal_account.vault_token_account,
            &withdrawal_account.user_token_account,
            withdrawal_ratio,
            user,
            vault,
            signer_seeds,
            token_program,
            associated_token_program,
            system_program,
        )?;
    }

    Ok(())
}

/// Calculate the withdrawal ratio based on the amount of vault tokens being withdrawn
/// and the total supply of vault tokens in NAV decimals
fn calculate_withdrawal_ratio(amount: u64, total_supply: u64) -> Result<u64> {
    amount
        .checked_mul(PRECISION)
        .ok_or(error!(VaultError::NumericOverflow))?
        .checked_div(total_supply)
        .ok_or(error!(VaultError::NumericOverflow))
        .map(|result| result as u64)
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
    vault_key: Pubkey,
) -> Result<Vec<WithdrawalAccounts<'info>>> {
    let mut withdrawal_accounts = Vec::new();

    for chunk in remaining_accounts.chunks(3) {
        // Each chunk must contain 3 accounts in this order:
        // 1. Token mint
        // 2. Vault token account
        // 3. User token account
        let mint = Account::<'info, Mint>::try_from(&chunk[0])?;
        let vault_token_account: Account<'info, TokenAccount> = Account::try_from(&chunk[1])?;
        let user_token_account: Account<'info, TokenAccount> = Account::try_from(&chunk[2])?;

        require!(
            whitelist.is_whitelisted(mint.key()),
            WhitelistError::TokenNotWhitelisted
        );

        require!(
            mint.key() == vault_token_account.mint,
            VaultError::MintAndTokenAccountMismatch
        );
        require!(
            vault_key == vault_token_account.owner,
            VaultError::VaultAndTokenAccountMismatch
        );

        // TODO: What happens if the user token account doesn't exist yet?
        require!(
            mint.key() == user_token_account.mint,
            VaultError::MintAndTokenAccountMismatch
        );
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
    user: &Signer<'info>,
    vault: &Account<'info, Vault>,
    signer_seeds: &[&[&[u8]]],
    token_program: &Program<'info, Token>,
    associated_token_program: &Program<'info, AssociatedToken>,
    system_program: &Program<'info, System>,
) -> Result<()> {
    // TODO: Throw error if to account (user token account) key doesn't match
    //       the one associated with the user and the mint
    // require!(...)

    // Create a token account for the recipient if it doesn't exist
    create_associated_token_account(
        user,
        mint,
        to,
        token_program,
        system_program,
        associated_token_program,
    )?;

    // Calculate the amount of tokens to withdraw based on the withdrawal ratio
    let token_balance = from.amount;
    let amount = calculate_token_withdrawal_amount(token_balance, withdrawal_ratio)?;

    // Transfer tokens from vault to withdrawer
    transfer_token_with_signer(
        from,
        to,
        amount,
        &vault.to_account_info(),
        signer_seeds,
        token_program,
    )?;

    Ok(())
}

/// Calculate the amount of tokens to withdraw for a specific
/// token balance and withdrawal ratio in NAV decimals
fn calculate_token_withdrawal_amount(token_balance: u64, withdrawal_ratio: u64) -> Result<u64> {
    token_balance
        .checked_mul(withdrawal_ratio)
        .ok_or(error!(VaultError::NumericOverflow))?
        .checked_div(PRECISION)
        .ok_or(error!(VaultError::NumericOverflow))
}
