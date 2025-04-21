use anchor_lang::prelude::*;

use crate::instructions::pause_starke::PauseOrResumeStarke;

pub fn _resume_starke(ctx: Context<PauseOrResumeStarke>) -> Result<()> {
    ctx.accounts.starke_config.update_pause_status(false)?;

    Ok(())
}
