use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::state::{
    StarkeConfig, StarkeConfigError, TokenWhitelist, UserDepositInfo, Vault, VaultError,
    WithdrawalRequest, WithdrawalRequested,
};

pub fn _request_withdrawal<'info>(
    ctx: Context<'_, '_, 'info, 'info, RequestWithdrawal<'info>>,
    amount: u64,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    require!(
        ctx.accounts.vault.withdrawal_delay_seconds > 0,
        VaultError::WithdrawalDelayRequired
    );

    require!(amount > 0, VaultError::InvalidAmount);

    require!(
        ctx.accounts.user_vtoken_account.amount >= amount,
        VaultError::InsufficientFunds
    );

    // Check lock-in period (same as withdraw)
    if !ctx.accounts.user_deposit_info.data_is_empty() {
        match UserDepositInfo::try_deserialize(&mut &ctx.accounts.user_deposit_info.data.borrow()[..])
        {
            Ok(user_deposit_info) => {
                if !user_deposit_info.is_lock_in_expired(ctx.accounts.clock.unix_timestamp) {
                    return Err(VaultError::LockInPeriodNotExpired.into());
                }
            }
            Err(_) => {}
        }
    }

    let bump = ctx.bumps.withdrawal_request;
    let ts = ctx.accounts.clock.unix_timestamp;

    ctx.accounts.withdrawal_request.user = ctx.accounts.user.key();
    ctx.accounts.withdrawal_request.vault = ctx.accounts.vault.key();
    ctx.accounts.withdrawal_request.vtoken_mint = ctx.accounts.vtoken_mint.key();
    ctx.accounts.withdrawal_request.amount = amount;
    ctx.accounts.withdrawal_request.requested_at = ts;
    ctx.accounts.withdrawal_request.bump = bump;

    emit!(WithdrawalRequested {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        amount,
        requested_at: ts,
    });

    msg!(
        "Withdrawal requested: {} vtokens, executable after {} seconds",
        amount,
        ctx.accounts.vault.withdrawal_delay_seconds
    );

    Ok(())
}

#[derive(Accounts)]
pub struct RequestWithdrawal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: Manager, used for vault PDA
    pub manager: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = vtoken_mint,
        associated_token::token_program = token_2022_program,
    )]
    pub user_vtoken_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        seeds = [Vault::VTOKEN_MINT_SEED, vault.key().as_ref()],
        bump = vault.mint_bump,
        mint::token_program = token_2022_program,
    )]
    pub vtoken_mint: InterfaceAccount<'info, Mint>,

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

    /// CHECK: Optional user deposit info for lock-in check
    pub user_deposit_info: UncheckedAccount<'info>,

    #[account(
        mut,
        init_if_needed,
        payer = user,
        space = WithdrawalRequest::MAX_SPACE,
        seeds = [WithdrawalRequest::SEED, vault.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub withdrawal_request: Account<'info, WithdrawalRequest>,

    pub clock: Sysvar<'info, Clock>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
}
