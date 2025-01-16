use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::controllers::{
    burn_vault_token, calculate_tokens_to_withdraw, transfer_token_with_signer,
};
use crate::state::{TokenWhitelist, Vault, WithdrawMade};

pub fn _withdraw<'info>(
    ctx: Context<'_, '_, 'info, 'info, Withdraw<'info>>,
    amount: u64,
) -> Result<()> {
    // TODO: Create a wrapper function for this in utils
    let manager = ctx.accounts.manager.key();
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    let signer_seeds = &[&vault_seeds[..]];

    // Burn vault tokens from depositor
    burn_vault_token(
        &ctx.accounts.user,
        &ctx.accounts.vault_token_mint,
        &ctx.accounts.user_vault_token_account,
        amount,
        signer_seeds,
        &ctx.accounts.token_program,
    )?;

    // First, calculate the total NAV (Net Asset Value) of the vault
    let total_nav = ctx.accounts.vault.get_nav(
        ctx.remaining_accounts,
        &ctx.accounts.whitelist,
        ctx.accounts.vault.key(),
    )?;

    // Calculate how many tokens to withdraw based on:
    let tokens_to_withdraw =
        calculate_tokens_to_withdraw(total_nav, amount, ctx.accounts.vault_token_mint.supply)?;

    // Transfer deposit tokens from vault to depositor
    transfer_token_with_signer(
        &ctx.accounts.vault_deposit_token_account,
        &ctx.accounts.user_deposit_token_account,
        tokens_to_withdraw,
        &ctx.accounts.vault.to_account_info(),
        signer_seeds,
        &ctx.accounts.token_program,
    )?;

    emit!(WithdrawMade {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        amount: tokens_to_withdraw,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

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

    // Vault's deposit token account (from account)
    #[account(
        mut,
        associated_token::authority = vault,
        associated_token::mint = deposit_token_mint,
    )]
    pub vault_deposit_token_account: Box<Account<'info, TokenAccount>>,

    // Depositor's deposit token account (to account)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = user,
        associated_token::mint = deposit_token_mint,
    )]
    pub user_deposit_token_account: Box<Account<'info, TokenAccount>>,

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

    // Deposit token mint
    #[account(
        constraint = deposit_token_mint.key() == vault.deposit_token_mint,
    )]
    pub deposit_token_mint: Box<Account<'info, Mint>>,

    // Deposit token price update
    pub price_update: Box<Account<'info, PriceUpdateV2>>,

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
