use anchor_lang::prelude::*;
use anchor_spl::{associated_token::*, token::*};

use crate::controllers::*;
use crate::state::*;

pub fn _deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    // TODO: Calculate the NAV and the deposit token value in USD so
    //       that we can mint the correct amount of vault tokens.
    //       We need to calculate the NAV before the transfer of the tokens.

    // Transfer deposit tokens from depositor to vault
    transfer_token(
        ctx.accounts.user_deposit_token_account.clone(),
        ctx.accounts.vault_deposit_token_account.clone(),
        amount,
        ctx.accounts.user.to_account_info(),
        ctx.accounts.token_program.clone(),
    )?;

    // Mint vault tokens to depositor
    let manager = ctx.accounts.manager.key();
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    let signer_seeds = &[&vault_seeds[..]];
    mint_vault_token(
        ctx.accounts.vault.clone(),
        ctx.accounts.vault_token_mint.clone(),
        ctx.accounts.vault_token_account.clone(),
        amount,
        signer_seeds,
        ctx.accounts.token_program.clone(),
    )?;

    emit!(DepositMade {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        amount,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    // Depositor
    #[account(mut)]
    pub user: Signer<'info>,

    // Manager
    /// CHECK: We can skip checking the manager
    pub manager: UncheckedAccount<'info>,

    // Depositor's deposit token account (from account)
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = deposit_token_mint,
    )]
    pub user_deposit_token_account: Account<'info, TokenAccount>,

    // Vault's deposit token account (to account)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = vault,
        associated_token::mint = deposit_token_mint,
    )]
    pub vault_deposit_token_account: Account<'info, TokenAccount>,

    // Depositor's vault token account
    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = user,
        associated_token::mint = vault_token_mint,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    // TODO: Consider whether we should boxed accounts everywhere
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

    // Deposit token mint
    #[account(
        constraint = deposit_token_mint.key() == vault.deposit_token_mint,
    )]
    pub deposit_token_mint: Box<Account<'info, Mint>>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
