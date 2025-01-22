use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::controllers::{burn_vault_token, withdraw_all_tokens};
use crate::state::{TokenWhitelist, Vault, WithdrawMade};

pub fn _withdraw<'info>(
    ctx: Context<'_, '_, 'info, 'info, Withdraw<'info>>,
    amount: u64,
) -> Result<()> {
    msg!("Withdraw instruction called with amount: {}", amount);

    let manager = ctx.accounts.manager.key();
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    let signer_seeds = &[&vault_seeds[..]];
    msg!("Vault seeds generated");

    // Burn vault tokens from depositor
    msg!("Burning vault tokens: {}", amount);
    burn_vault_token(
        &ctx.accounts.user,
        &ctx.accounts.vault_token_mint,
        &ctx.accounts.user_vault_token_account,
        amount,
        signer_seeds,
        &ctx.accounts.token_program,
    )?;
    msg!("Vault tokens burned successfully");

    // Process withdrawals for all tokens in the vault
    msg!("Processing withdrawals for all vault tokens");
    withdraw_all_tokens(
        &ctx.remaining_accounts,
        &ctx.accounts.user,
        &ctx.accounts.vault,
        &ctx.accounts.vault_token_mint,
        amount,
        &ctx.accounts.whitelist,
        signer_seeds,
        &ctx.accounts.token_program,
        &ctx.accounts.associated_token_program,
        &ctx.accounts.system_program,
    )?;
    msg!("All token withdrawals processed successfully");

    emit!(WithdrawMade {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        amount,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    msg!("Withdraw event emitted");

    Ok(())
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // Depositor
    #[account(mut)]
    pub user: Signer<'info>,

    // Manager
    /// CHECK: We can skip checking the manager
    pub manager: UncheckedAccount<'info>,

    // Depositor's vault token account
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = vault_token_mint,
    )]
    pub user_vault_token_account: Box<Account<'info, TokenAccount>>,

    // Vault
    #[account(
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    // Vault token mint
    #[account(
        mut,
        seeds = [Vault::VAULT_TOKEN_MINT_SEED, vault.key().as_ref()],
        bump = vault.mint_bump,
    )]
    pub vault_token_mint: Box<Account<'info, Mint>>,

    // Token whitelist
    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Box<Account<'info, TokenWhitelist>>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
