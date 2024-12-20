use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::controllers::{
    compute_token_value_usd, get_token_price_from_pyth_feed, mint_vault_token, transfer_token,
    transform_price_to_nav_decimals,
};
use crate::state::{DepositMade, TokenWhitelist, Vault, VaultError};

pub fn _deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, Deposit<'info>>,
    amount: u64,
) -> Result<()> {
    // Calculate the total NAV using vault's get_nav function
    let total_nav = ctx.accounts.vault.get_nav(
        ctx.remaining_accounts,
        ctx.accounts.whitelist.clone(),
        ctx.accounts.vault.key(),
        ctx.accounts.price_update.clone(),
    )?;

    // Calculate the USD value of deposit tokens
    let deposit_price_feed_id = ctx
        .accounts
        .whitelist
        .get_price_feed_id(&ctx.accounts.deposit_token_mint.key())?;

    let deposit_price =
        get_token_price_from_pyth_feed(deposit_price_feed_id, ctx.accounts.price_update.clone())?;

    let deposit_price_in_nav_decimals = transform_price_to_nav_decimals(deposit_price)?;
    let deposit_value = compute_token_value_usd(
        amount,
        ctx.accounts.deposit_token_mint.decimals,
        deposit_price_in_nav_decimals,
    )?;

    // Calculate vault tokens to mint based on NAV
    let vault_tokens_to_mint = if total_nav == 0 {
        // Initial deposit - mint 1:1
        amount
    } else {
        // Calculate proportional amount based on NAV
        let supply = ctx.accounts.vault_token_mint.supply;
        (deposit_value as u128)
            .checked_mul(supply as u128)
            .ok_or(error!(VaultError::NumericOverflow))?
            .checked_div(total_nav as u128)
            .ok_or(error!(VaultError::NumericOverflow))? as u64
    };

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
        vault_tokens_to_mint,
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

    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Box<Account<'info, TokenWhitelist>>,

    pub price_update: Box<Account<'info, PriceUpdateV2>>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
