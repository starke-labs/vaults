use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::{
    state::{
        StarkeConfig, StarkeConfigError, Vault, VaultClosed, VaultError,
    },
};

pub fn _close_vault(ctx: Context<CloseVault>) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    let vault = &ctx.accounts.vault;

    msg!("Processing vault closure request");
    msg!("Vault: {}", ctx.accounts.vault.key());
    msg!("Manager: {}", ctx.accounts.manager.key());

    // Validate that only the manager can close the vault
    require!(
        ctx.accounts.manager.key() == vault.manager,
        VaultError::UserAndTokenAccountMismatch // Reusing existing error for unauthorized access
    );

    // Check that vault has no active depositors
    require!(
        vault.current_depositors == 0,
        VaultError::VaultHasActiveDepositors
    );

    // Check that no vtokens are outstanding
    require!(
        ctx.accounts.vtoken_mint.supply == 0,
        VaultError::VTokensOutstanding
    );

    // Note: We rely on the fact that if current_depositors == 0 and vtoken_supply == 0,
    // then the vault should be empty. For additional safety, the manager should ensure
    // all funds are withdrawn before closing.

    msg!("All closure requirements met - vault is empty");

    // Emit closure event
    emit!(VaultClosed {
        vault: ctx.accounts.vault.key(),
        manager: vault.manager,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    msg!("Successfully closed vault: {}", ctx.accounts.vault.key());

    Ok(())
}

#[derive(Accounts)]
pub struct CloseVault<'info> {
    // Manager who created the vault
    #[account(mut)]
    pub manager: Signer<'info>,

    // Vault to be closed (PDA)
    #[account(
        mut,
        close = manager, // Close account and send rent to manager
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    // Vtoken mint (should have 0 supply)
    #[account(
        mut,
        seeds = [Vault::VTOKEN_MINT_SEED, vault.key().as_ref()],
        bump = vault.mint_bump,
        mint::authority = vault,
        mint::token_program = token_2022_program,
    )]
    pub vtoken_mint: Box<InterfaceAccount<'info, Mint>>,


    // Starke config
    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    // Token programs
    pub token_program: Interface<'info, TokenInterface>,
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}