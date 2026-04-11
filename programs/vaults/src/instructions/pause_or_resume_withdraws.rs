use anchor_lang::prelude::*;

use crate::state::{StarkeConfig, Vault, VaultError};

pub fn _pause_withdraws(ctx: Context<PauseOrResumeWithdraws>) -> Result<()> {
    msg!(
        "Processing withdraws pause request for vault: {} by manager: {}",
        ctx.accounts.vault.key(),
        ctx.accounts.manager.key()
    );

    // Check program pause
    require!(
        !ctx.accounts.starke_config.is_paused,
        crate::state::StarkeConfigError::StarkePaused
    );

    // Pause withdraws
    ctx.accounts.vault.pause_withdraws();
    Ok(())
}

pub fn _resume_withdraws(ctx: Context<PauseOrResumeWithdraws>) -> Result<()> {
    msg!(
        "Processing withdraws resume request for vault: {} by manager: {}",
        ctx.accounts.vault.key(),
        ctx.accounts.manager.key()
    );

    // Check program pause
    require!(
        !ctx.accounts.starke_config.is_paused,
        crate::state::StarkeConfigError::StarkePaused
    );

    // Resume withdraws
    ctx.accounts.vault.resume_withdraws();
    Ok(())
}

#[derive(Accounts)]
pub struct PauseOrResumeWithdraws<'info> {
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
