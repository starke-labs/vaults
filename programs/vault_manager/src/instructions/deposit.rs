use anchor_lang::prelude::*;
use anchor_spl::token::*;

use crate::state::*;

pub fn _deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    // Transfer deposit tokens from depositor to vault
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let transfer_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.vault_deposit_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program.clone(), transfer_accounts);
    transfer(cpi_ctx, amount)?;

    // Mint vault tokens to depositor
    let manager = ctx.accounts.manager.key();
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    let signer = &[&vault_seeds[..]];
    let mint_accounts = MintTo {
        mint: ctx.accounts.vault_token_mint.to_account_info(),
        to: ctx.accounts.user_vault_token_account.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, mint_accounts, signer);
    mint_to(cpi_ctx, amount)?;

    // Update vault total deposits
    ctx.accounts.vault.deposit(amount)?;

    emit!(DepositMade {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        amount,
        // TODO: fix this
        total_deposited: ctx.accounts.vault.total_deposits,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Deposit<'info> {
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
