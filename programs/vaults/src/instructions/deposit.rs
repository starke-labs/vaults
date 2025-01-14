use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::controllers::{
    calculate_deposit_token_value, calculate_vault_tokens_to_mint, mint_vault_token, transfer_token,
};
use crate::state::{DepositMade, TokenWhitelist, Vault};

pub fn _deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, Deposit<'info>>,
    amount: u64,
) -> Result<()> {
    // Calculate the total NAV using vault's get_nav function
    let total_nav = ctx.accounts.vault.get_nav(
        ctx.remaining_accounts,
        ctx.accounts.whitelist.clone(),
        ctx.accounts.vault.key(),
    )?;
    msg!("Total NAV: {}", total_nav);

    // Calculate the USD value of deposit tokens
    let deposit_value = calculate_deposit_token_value(
        &ctx.accounts.whitelist,
        &ctx.accounts.deposit_token_mint.key(),
        ctx.accounts.deposit_token_mint.decimals,
        amount,
        ctx.accounts.deposit_token_price_update.clone(),
    )?;
    msg!("Deposit value: {}", deposit_value);

    // Calculate vault tokens to mint based on NAV
    let vault_tokens_to_mint = calculate_vault_tokens_to_mint(
        total_nav,
        deposit_value,
        ctx.accounts.vault_token_mint.supply,
    )?;
    msg!("Vault tokens to mint: {}", vault_tokens_to_mint);

    // Commenting this out for testing
    // // Transfer deposit tokens from depositor to vault
    // transfer_token(
    //     ctx.accounts.user_deposit_token_account.clone(),
    //     ctx.accounts.vault_deposit_token_account.clone(),
    //     amount,
    //     ctx.accounts.user.to_account_info(),
    //     ctx.accounts.token_program.clone(),
    // )?;

    // // Mint vault tokens to depositor
    // let manager = ctx.accounts.manager.key();
    // let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    // let signer_seeds = &[&vault_seeds[..]];
    // mint_vault_token(
    //     ctx.accounts.vault.clone(),
    //     ctx.accounts.vault_token_mint.clone(),
    //     ctx.accounts.vault_token_account.clone(),
    //     vault_tokens_to_mint,
    //     signer_seeds,
    //     ctx.accounts.token_program.clone(),
    // )?;

    // emit!(DepositMade {
    //     vault: ctx.accounts.vault.key(),
    //     user: ctx.accounts.user.key(),
    //     amount,
    //     timestamp: ctx.accounts.clock.unix_timestamp,
    // });

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
    pub user_deposit_token_account: Box<Account<'info, TokenAccount>>,

    // Vault's deposit token account (to account)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = vault,
        associated_token::mint = deposit_token_mint,
    )]
    pub vault_deposit_token_account: Box<Account<'info, TokenAccount>>,

    // Depositor's vault token account
    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = user,
        associated_token::mint = vault_token_mint,
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,

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
    pub deposit_token_price_update: Box<Account<'info, PriceUpdateV2>>,

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
