use anchor_lang::prelude::*;

use crate::state::*;

pub fn _update_vault_fees(
    ctx: Context<UpdateVaultFees>,
    new_entry_fee: u16,
    new_exit_fee: u16,
) -> Result<()> {
    ctx.accounts.vault.update_fees(
        new_entry_fee,
        new_exit_fee,
        ctx.accounts.clock.unix_timestamp,
    )?;

    emit!(VaultFeesUpdateRequested {
        vault: ctx.accounts.vault.key(),
        manager: ctx.accounts.manager.key(),
        pending_entry_fee: new_entry_fee,
        pending_exit_fee: new_exit_fee,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateVaultFees<'info> {
    pub manager: Signer<'info>,

    #[account(
        mut,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub clock: Sysvar<'info, Clock>,
}
