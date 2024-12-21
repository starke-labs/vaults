use anchor_lang::prelude::*;
use anchor_spl::token::{burn, mint_to, Burn, Mint, MintTo, Token, TokenAccount};

use crate::state::{Vault, VaultError};

pub fn mint_vault_token<'info>(
    vault: Box<Account<'info, Vault>>,
    mint: Box<Account<'info, Mint>>,
    to: Box<Account<'info, TokenAccount>>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
    token_program: Program<'info, Token>,
) -> Result<()> {
    let mint_accounts = MintTo {
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: vault.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(token_program.to_account_info(), mint_accounts, signer_seeds);
    mint_to(cpi_ctx, amount)?;

    Ok(())
}

pub fn burn_vault_token<'info>(
    user: Signer<'info>,
    mint: Box<Account<'info, Mint>>,
    from: Box<Account<'info, TokenAccount>>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
    token_program: Program<'info, Token>,
) -> Result<()> {
    let burn_accounts = Burn {
        mint: mint.to_account_info(),
        from: from.to_account_info(),
        authority: user.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(token_program.to_account_info(), burn_accounts, signer_seeds);
    burn(cpi_ctx, amount)?;

    Ok(())
}

pub fn calculate_vault_tokens_to_mint(
    total_nav: u64,
    deposit_value: u64,
    vault_token_supply: u64,
) -> Result<u64> {
    if total_nav == 0 {
        // Initial deposit - mint 1:1
        Ok(deposit_value)
    } else {
        // Calculate proportional amount based on NAV
        (deposit_value as u128)
            .checked_mul(vault_token_supply as u128)
            .ok_or(error!(VaultError::NumericOverflow))?
            .checked_div(total_nav as u128)
            .ok_or(error!(VaultError::NumericOverflow))
            .map(|result| result as u64)
    }
}

pub fn calculate_tokens_to_withdraw(
    total_nav: u64,
    vault_tokens_to_burn: u64,
    vault_token_supply: u64,
) -> Result<u64> {
    (vault_tokens_to_burn as u128)
        .checked_mul(total_nav as u128)
        .ok_or(error!(VaultError::NumericOverflow))?
        .checked_div(vault_token_supply as u128)
        .ok_or(error!(VaultError::NumericOverflow))
        .map(|result| result as u64)
}
