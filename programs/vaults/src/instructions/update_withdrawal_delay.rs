use anchor_lang::prelude::*;

use crate::state::{StarkeConfig, Vault, VaultError};

pub fn _update_withdrawal_delay(
    ctx: Context<UpdateWithdrawalDelay>,
    withdrawal_delay_seconds: u64,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        crate::state::StarkeConfigError::StarkePaused
    );

    require!(
        withdrawal_delay_seconds <= Vault::MAX_WITHDRAWAL_DELAY_SECONDS,
        VaultError::InvalidWithdrawalDelay
    );

    ctx.accounts.vault.withdrawal_delay_seconds = withdrawal_delay_seconds;

    msg!(
        "Withdrawal delay updated to {} seconds for vault: {}",
        withdrawal_delay_seconds,
        ctx.accounts.vault.key()
    );

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateWithdrawalDelay<'info> {
    pub manager: Signer<'info>,

    #[account(
        mut,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump,
        constraint = vault.manager == manager.key() @ VaultError::Unauthorized
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,
}
