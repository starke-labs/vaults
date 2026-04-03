use anchor_lang::prelude::*;

use crate::state::{StarkeConfig, Vault, VaultError};

pub fn _pause_deposits(ctx: Context<PauseOrResumeDeposits>) -> Result<()> {
    msg!(
        "Processing deposits pause request for vault: {} by manager: {}",
        ctx.accounts.vault.key(),
        ctx.accounts.manager.key()
    );

    // Check program pause
    require!(
        !ctx.accounts.starke_config.is_paused,
        crate::state::StarkeConfigError::StarkePaused
    );

    // Pause deposits
    ctx.accounts.vault.pause_deposits();
    Ok(())
}

pub fn _resume_deposits(ctx: Context<PauseOrResumeDeposits>) -> Result<()> {
    msg!(
        "Processing deposits resume request for vault: {} by manager: {}",
        ctx.accounts.vault.key(),
        ctx.accounts.manager.key()
    );

    // Check program pause
    require!(
        !ctx.accounts.starke_config.is_paused,
        crate::state::StarkeConfigError::StarkePaused
    );

    // Resume deposits
    ctx.accounts.vault.resume_deposits();
    Ok(())
}

#[derive(Accounts)]
pub struct PauseOrResumeDeposits<'info> {
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
