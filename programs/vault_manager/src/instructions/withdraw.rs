use anchor_lang::prelude::*;
use anchor_spl::token::*;

use crate::state::*;

pub fn _withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    // Burn vault tokens from depositor
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let burn_accounts = Burn {
        mint: ctx.accounts.vault_token_mint.to_account_info(),
        from: ctx.accounts.user_vault_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program.clone(), burn_accounts);
    burn(cpi_ctx, amount)?;

    // Transfer deposit tokens from vault to depositor
    let manager = ctx.accounts.manager.key();
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    let signer = &[&vault_seeds[..]];
    let transfer_accounts = Transfer {
        from: ctx.accounts.vault_deposit_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, transfer_accounts, signer);
    transfer(cpi_ctx, amount)?;

    // Update vault total deposits
    ctx.accounts.vault.withdraw(amount)?;

    emit!(WithdrawMade {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        amount,
        remaining_balance: ctx.accounts.vault.total_deposits,
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
