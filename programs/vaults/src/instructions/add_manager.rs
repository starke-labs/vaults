use anchor_lang::prelude::*;

use crate::constants::STARKE_AUTHORITY;
use crate::state::{ManagerWhitelist, ManagerWhitelistError, WhitelistManagerAdded};

pub fn _add_manager(ctx: Context<ModifyManagerWhitelist>, manager_pubkey: &Pubkey) -> Result<()> {
    msg!("Processing request to add manager: {}", manager_pubkey);

    ctx.accounts.manager_whitelist.add_manager(manager_pubkey)?;

    msg!("Successfully added manager to whitelist");

    emit!(WhitelistManagerAdded {
        manager: *manager_pubkey,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct ModifyManagerWhitelist<'info> {
    #[account(address = STARKE_AUTHORITY @ ManagerWhitelistError::UnauthorizedAccess)]
    pub starke_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [ManagerWhitelist::SEED],
        bump = manager_whitelist.bump,
    )]
    pub manager_whitelist: Box<Account<'info, ManagerWhitelist>>,

    pub clock: Sysvar<'info, Clock>,
}
