use anchor_lang::prelude::*;

use crate::instructions::add_token::ModifyTokenWhitelist;
use crate::state::WhitelistTokenRemoved;

pub fn _remove_token(ctx: Context<ModifyTokenWhitelist>, token: Pubkey) -> Result<()> {
    msg!("Processing request to remove token: {}", token);

    ctx.accounts.token_whitelist.remove_token(&token)?;

    msg!("Successfully removed token from whitelist");

    emit!(WhitelistTokenRemoved {
        mint: token,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}
