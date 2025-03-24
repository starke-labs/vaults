use anchor_lang::prelude::*;

use crate::instructions::add_token::ModifyWhitelist;
use crate::state::WhitelistTokenRemoved;

pub fn _remove_token(ctx: Context<ModifyWhitelist>, token: &Pubkey) -> Result<()> {
    msg!("Processing request to remove token: {}", token);

    ctx.accounts.whitelist.remove_token(token)?;

    msg!("Successfully removed token from whitelist");

    emit!(WhitelistTokenRemoved {
        mint: *token,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}
