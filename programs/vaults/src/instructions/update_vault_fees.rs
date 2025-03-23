use anchor_lang::prelude::*;

use crate::state::{Vault, VaultFeesUpdateRequested};

pub fn _update_vault_fees(
    ctx: Context<UpdateVaultFees>,
    new_entry_fee: u16,
    new_exit_fee: u16,
) -> Result<()> {
    msg!("Processing fee update request");
    msg!("Vault: {}", ctx.accounts.vault.key());
    msg!("Manager: {}", ctx.accounts.manager.key());
    msg!(
        "Current entry fee: {}, exit fee: {}",
        ctx.accounts.vault.entry_fee,
        ctx.accounts.vault.exit_fee
    );
    msg!(
        "New entry fee: {}, exit fee: {}",
        new_entry_fee,
        new_exit_fee
    );

    ctx.accounts.vault.update_fees(
        new_entry_fee,
        new_exit_fee,
        ctx.accounts.clock.unix_timestamp,
    )?;

    msg!("Fees updated successfully");

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
