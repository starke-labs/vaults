use anchor_lang::prelude::*;
use anchor_spl::token::*;

use crate::controllers::*;
use crate::state::*;

pub fn _withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    // Burn vault tokens from depositor
    burn_vault_token(
        ctx.accounts.vault.clone(),
        ctx.accounts.vault_token_mint.clone(),
        ctx.accounts.user_vault_token_account.clone(),
        amount,
        ctx.accounts.token_program.clone(),
    )?;

    // Transfer deposit tokens from vault to depositor
    // TODO: Create a wrapper function for this in utils
    let manager = ctx.accounts.manager.key();
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    let signer_seeds = &[&vault_seeds[..]];
    transfer_token_with_signer(
        ctx.accounts.vault_deposit_token_account.clone(),
        ctx.accounts.user_token_account.clone(),
        amount,
        ctx.accounts.vault.to_account_info(),
        signer_seeds,
        ctx.accounts.token_program.clone(),
    )?;

    emit!(WithdrawMade {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        amount,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // Depositor
    #[account(mut)]
    pub user: Signer<'info>,

    // Depositor's deposit token account
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == vault.deposit_token,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    // Depositor's vault token account
    #[account(
        mut,
        constraint = user_vault_token_account.owner == user.key(),
        constraint = user_vault_token_account.mint == vault_token_mint.key(),
    )]
    pub user_vault_token_account: Account<'info, TokenAccount>,

    // Manager
    /// CHECK: We can skip checking the manager
    pub manager: UncheckedAccount<'info>,

    // Vault
    #[account(
        mut,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,

    // Vault's deposit token account
    #[account(
        mut,
        constraint = vault_deposit_token_account.owner == vault.key(),
        constraint = vault_deposit_token_account.mint == vault.deposit_token,
    )]
    pub vault_deposit_token_account: Account<'info, TokenAccount>,

    // Vault's token mint
    #[account(
        mut,
        constraint = vault_token_mint.key() == vault.vault_token_mint,
    )]
    pub vault_token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}
