use anchor_lang::prelude::*;

use crate::{constants::STARKE_AUTHORITY, state::StarkeConfig, state::StarkeConfigError};

pub fn _pause_starke(ctx: Context<PauseOrResumeStarke>) -> Result<()> {
    ctx.accounts.starke_config.update_pause_status(true)?;
    Ok(())
}
pub fn _resume_starke(ctx: Context<PauseOrResumeStarke>) -> Result<()> {
    ctx.accounts.starke_config.update_pause_status(false)?;
    Ok(())
}

#[derive(Accounts)]
pub struct PauseOrResumeStarke<'info> {
    #[account(mut, constraint = authority.key() == STARKE_AUTHORITY @ StarkeConfigError::Unauthorized)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,
}
