use anchor_lang::prelude::*;

use crate::{instructions::add_manager::ModifyManagerWhitelist, state::WhitelistManagerRemoved};

pub fn _remove_manager(
    ctx: Context<ModifyManagerWhitelist>,
    manager_pubkey: Pubkey,
) -> Result<()> {
    msg!("Processing request to remove manager: {}", manager_pubkey);

    ctx.accounts
        .manager_whitelist
        .remove_manager(&manager_pubkey)?;

    msg!("Successfully removed manager from whitelist");

    emit!(WhitelistManagerRemoved {
        manager: manager_pubkey,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}
