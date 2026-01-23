use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::STARKE_AUTHORITY;
use crate::controllers::{burn_vtoken, withdraw_all_tokens};
use crate::state::{
    StarkeConfig, StarkeConfigError, TokenWhitelist, TokenWhitelistError, Vault, VaultError,
    WithdrawalRequest, Withdrawn,
};

pub fn _complete_withdrawal<'info>(
    ctx: Context<'_, '_, 'info, 'info, CompleteWithdrawal<'info>>,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    let req = &ctx.accounts.withdrawal_request;
    let amount = req.amount;

    require!(amount > 0, VaultError::InvalidAmount);

    let delay = ctx.accounts.vault.withdrawal_delay_seconds as i64;
    let earliest = req
        .requested_at
        .checked_add(delay)
        .ok_or(VaultError::NumericOverflow)?;
    require!(
        ctx.accounts.clock.unix_timestamp >= earliest,
        VaultError::WithdrawalDelayNotExpired
    );

    require!(
        ctx.accounts.user.key() == req.user,
        VaultError::WithdrawalRequestNotFound
    );
    require!(
        ctx.accounts.vault.key() == req.vault,
        VaultError::WithdrawalRequestNotFound
    );
    require!(
        ctx.accounts.vtoken_mint.key() == req.vtoken_mint,
        VaultError::WithdrawalRequestNotFound
    );

    require!(
        ctx.accounts.user_vtoken_account.amount >= amount,
        VaultError::InsufficientFunds
    );

    let manager = ctx.accounts.manager.key();
    let signer_seeds: &[&[&[u8]]] = &[&[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]]];

    let user_balance_before = ctx.accounts.user_vtoken_account.amount;
    let will_be_zero_balance = user_balance_before == amount;

    burn_vtoken(
        &ctx.accounts.user,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.user_vtoken_account,
        amount,
        signer_seeds,
        &ctx.accounts.token_2022_program,
    )?;

    if will_be_zero_balance {
        ctx.accounts.vault.decrement_depositor_count()?;
    }

    withdraw_all_tokens(
        ctx.remaining_accounts,
        &ctx.accounts.user,
        &ctx.accounts.vault,
        &ctx.accounts.vtoken_mint,
        amount,
        &ctx.accounts.token_whitelist,
        signer_seeds,
        &ctx.accounts.associated_token_program,
        &ctx.accounts.system_program,
    )?;

    emit!(Withdrawn {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        vtoken_burned_amount: amount,
        new_vtoken_supply: ctx.accounts.vtoken_mint.supply.checked_sub(amount).unwrap(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    msg!("Delayed withdrawal completed successfully");

    Ok(())
}

#[derive(Accounts)]
pub struct CompleteWithdrawal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        address = STARKE_AUTHORITY @ TokenWhitelistError::UnauthorizedAccess,
    )]
    pub authority: Signer<'info>,

    /// CHECK: Manager for vault PDA
    pub manager: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = vtoken_mint,
        associated_token::token_program = token_2022_program,
    )]
    pub user_vtoken_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        seeds = [Vault::VTOKEN_MINT_SEED, vault.key().as_ref()],
        bump = vault.mint_bump,
        mint::token_program = token_2022_program,
    )]
    pub vtoken_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = token_whitelist.bump,
    )]
    pub token_whitelist: Box<Account<'info, TokenWhitelist>>,

    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    #[account(
        mut,
        close = user,
        seeds = [WithdrawalRequest::SEED, vault.key().as_ref(), user.key().as_ref()],
        bump = withdrawal_request.bump,
        constraint = withdrawal_request.amount > 0 @ VaultError::WithdrawalRequestNotFound,
    )]
    pub withdrawal_request: Account<'info, WithdrawalRequest>,

    pub clock: Sysvar<'info, Clock>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
