use anchor_lang::prelude::*;
use anchor_spl::{token_2022::Token2022, token_interface::Mint};
use transfer_hook::{
    constants::EXTRA_ACCOUNT_METAS_SEED, program::TransferHook, state::VtokenConfig,
};

use crate::{
    controllers::close_vtoken_config,
    state::{StarkeConfig, Vault, VaultCloseError},
};

pub fn _close_vault(ctx: Context<CloseVault>) -> Result<()> {
    msg!(
        "Processing vault close request for vault: {} by manager: {}",
        ctx.accounts.vault.key(),
        ctx.accounts.manager.key()
    );

    // Check program pause
    require!(
        !ctx.accounts.starke_config.is_paused,
        crate::state::StarkeConfigError::StarkePaused
    );

    // Check supply
    require!(
        ctx.accounts.vtoken_mint.supply == 0,
        VaultCloseError::NonZeroVtokenSupply
    );

    // This cleans up the extra_account_metas and vtoken_config
    close_vtoken_config(
        &ctx.accounts.manager,
        &ctx.accounts.vtoken_config,
        &ctx.accounts.extra_accounts_meta,
        &ctx.accounts.transfer_hook_program,
        &ctx.accounts.vtoken_mint,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct CloseVault<'info> {
    // Vault manager
    #[account(mut)]
    pub manager: Signer<'info>,

    // Vault
    #[account(
        mut,
        close = manager,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump,
        constraint = vault.manager == manager.key() @ VaultCloseError::UnauthorizedManager
    )]
    pub vault: Box<Account<'info, Vault>>,

    // Vtoken mint
    #[account(
        mut,
        seeds = [Vault::VTOKEN_MINT_SEED, vault.key().as_ref()],
        bump,
        constraint = vtoken_mint.supply == 0 @ VaultCloseError::NonZeroVtokenSupply,
        mint::token_program = token_2022_program,
        mint::authority = vault,
        extensions::transfer_hook::program_id = transfer_hook_program,
        extensions::transfer_hook::authority = manager,
    )]
    pub vtoken_mint: Box<InterfaceAccount<'info, Mint>>,

    // Starke config
    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    /// CHECK: This account is closed in the close_extra_account_metas instruction
    /// and its data is structured according to spl_tlv_account_resolution::state::ExtraAccountMetaList.
    /// The Token2022 program validates this account during CPI.
    #[account(
        mut,
        seeds = [EXTRA_ACCOUNT_METAS_SEED, vtoken_mint.key().as_ref()],
        bump,
        seeds::program = transfer_hook_program.key(),
    )]
    pub extra_accounts_meta: AccountInfo<'info>,

    /// CHECK: This account will be closed by the transfer hook program
    #[account(
        mut,
        seeds = [VtokenConfig::SEED, vtoken_mint.key().as_ref()],
        bump,
        seeds::program = transfer_hook_program.key(),
    )]
    pub vtoken_config: AccountInfo<'info>,

    pub transfer_hook_program: Program<'info, TransferHook>,
    pub token_2022_program: Program<'info, Token2022>,
    pub clock: Sysvar<'info, Clock>,
}
