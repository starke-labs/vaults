use anchor_lang::prelude::*;

use crate::constants::PROGRAM_AUTHORITY;
use crate::state::{TokenWhitelist, WhitelistError, WhitelistTokenAdded};

pub fn _add_token(
    ctx: Context<ModifyWhitelist>,
    token: &Pubkey,
    price_feed_id: &str,
    price_update: &Pubkey,
) -> Result<()> {
    ctx.accounts
        .whitelist
        .add_token(&token, price_feed_id, &price_update)?;

    emit!(WhitelistTokenAdded {
        mint: *token,
        price_feed_id: price_feed_id.to_string(),
        price_update: *price_update,
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
        seeds = [TokenWhitelist::SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Box<Account<'info, TokenWhitelist>>,

    pub clock: Sysvar<'info, Clock>,
}
