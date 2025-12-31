use anchor_lang::prelude::*;

use crate::constants::STARKE_AUTHORITY;
use crate::state::{Vault, VaultError};

pub fn _update_platform_fees(
    ctx: Context<UpdatePlatformFees>,
    vault_manager: Pubkey,
    new_platform_fees_rate: u16,
) -> Result<()> {
    msg!("Processing request to update platform fees");
    msg!("Vault manager: {}", vault_manager);
    msg!("New platform fees rate: {} bps", new_platform_fees_rate);

    // Validate fee rate (max 100% = 10000 bps)
    require!(
        new_platform_fees_rate <= 10000,
        VaultError::InvalidFee
    );

    // Update the rate
    ctx.accounts.vault.platform_fees_rate = new_platform_fees_rate;

    msg!(
        "Successfully updated platform fees to {} bps",
        new_platform_fees_rate
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(vault_manager: Pubkey)]
pub struct UpdatePlatformFees<'info> {
    // Only STARKE_AUTHORITY can call this
    #[account(
        address = STARKE_AUTHORITY @ VaultError::Unauthorized,
    )]
    pub authority: Signer<'info>,

    // Vault to update
    #[account(
        mut,
        seeds = [Vault::SEED, vault_manager.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub clock: Sysvar<'info, Clock>,
}



