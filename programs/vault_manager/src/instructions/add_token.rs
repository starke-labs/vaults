use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::*;

pub fn _add_token(ctx: Context<ModifyWhitelist>, token: Pubkey) -> Result<()> {
    ctx.accounts.whitelist.add_token(token)?;

    emit!(TokenWhitelisted {
        token,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct ModifyWhitelist<'info> {
    #[account(
        address = PROGRAM_AUTHORITY @ WhitelistError::UnauthorizedAccess,
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [TOKEN_WHITELIST_SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Account<'info, TokenWhitelist>,

    pub clock: Sysvar<'info, Clock>,
}
