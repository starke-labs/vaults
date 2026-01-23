use anchor_lang::prelude::*;

use crate::state::{StarkeConfig, Vault, VaultError};

pub fn _update_lock_in_period(
    ctx: Context<UpdateLockInPeriod>,
    lock_in_period_seconds: Option<u64>,
) -> Result<()> {
    msg!(
        "Processing lock-in period update request for vault: {} by manager: {}",
        ctx.accounts.vault.key(),
        ctx.accounts.manager.key()
    );
    msg!(
        "New lock-in period: {:?} seconds",
        lock_in_period_seconds
    );

    // Check program pause
    require!(
        !ctx.accounts.starke_config.is_paused,
        crate::state::StarkeConfigError::StarkePaused
    );

    // Validate lock-in period if provided (must be one of the allowed periods)
    if let Some(lock_in) = lock_in_period_seconds {
        require!(
            lock_in == Vault::LOCK_IN_1_MONTH
                || lock_in == Vault::LOCK_IN_3_MONTHS
                || lock_in == Vault::LOCK_IN_6_MONTHS
                || lock_in == Vault::LOCK_IN_1_YEAR,
            VaultError::InvalidLockInPeriod
        );
    }

    // Update lock-in period
    // Note: This only affects new investors. Existing investors' lock-in periods
    // are stored in their UserDepositInfo accounts and won't change.
    ctx.accounts.vault.lock_in_period_seconds = lock_in_period_seconds;

    msg!(
        "Lock-in period updated successfully. New investors will be subject to: {:?} seconds",
        lock_in_period_seconds
    );

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateLockInPeriod<'info> {
    // Vault manager
    #[account(mut)]
    pub manager: Signer<'info>,

    // Vault
    #[account(
        mut,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump,
        constraint = vault.manager == manager.key() @ VaultError::Unauthorized
    )]
    pub vault: Box<Account<'info, Vault>>,

    // Starke config
    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,
}

