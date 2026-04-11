use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::Mint;
use transfer_hook::{
    cpi::{accounts::SetVtokenIsTransferrableAccounts, set_vtoken_is_transferrable},
    program::TransferHook,
    state::VtokenConfig,
};

use crate::state::{
    InvestorTier, InvestorType, InvestorTypeWithRange, StarkeConfig, Vault, VaultError,
};

#[allow(clippy::too_many_arguments)]
pub fn _update_fund(
    ctx: Context<UpdateFund>,
    max_allowed_aum: u64,
    allowed_investor_types: Vec<InvestorType>,
    allowed_investor_tiers: Vec<InvestorTier>,
    range_allowed_per_investor_type: Vec<InvestorTypeWithRange>,
    max_depositors: u32,
    management_fee_rate: u16,
    is_transferrable: bool,
) -> Result<()> {
    msg!(
        "Processing fund update request for vault: {} by manager: {}",
        ctx.accounts.vault.key(),
        ctx.accounts.manager.key()
    );

    // Check program pause
    require!(
        !ctx.accounts.starke_config.is_paused,
        crate::state::StarkeConfigError::StarkePaused
    );

    let accs = SetVtokenIsTransferrableAccounts {
        manager: ctx.accounts.manager.to_account_info(),
        mint: ctx.accounts.vtoken_mint.to_account_info(),
        vtoken_config: ctx.accounts.vtoken_config.to_account_info(),
    };
    set_vtoken_is_transferrable(
        CpiContext::new(ctx.accounts.transfer_hook_program.to_account_info(), accs),
        is_transferrable,
    )?;
    msg!("Set vtoken transferability to {}", is_transferrable);

    // Update vault
    ctx.accounts.vault.update_fund(
        max_allowed_aum,
        allowed_investor_types,
        allowed_investor_tiers,
        range_allowed_per_investor_type,
        max_depositors,
        management_fee_rate,
    )?;
    msg!("Updated vault fund");
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateFund<'info> {
    // Vault manager
    #[account(mut)]
    pub manager: Signer<'info>,

    // Vault
    #[account(
        mut,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump,
        constraint = vault.manager == manager.key() @ VaultError::Unauthorized
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        seeds = [Vault::VTOKEN_MINT_SEED, vault.key().as_ref()],
        bump = vault.mint_bump,
        mint::token_program = token_2022_program,
        constraint = vault.mint == vtoken_mint.key() @ VaultError::InvalidVtokenMint
    )]
    pub vtoken_mint: InterfaceAccount<'info, Mint>,

    // Starke config
    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    #[account(
        mut,
        seeds = [VtokenConfig::SEED, vtoken_mint.key().as_ref()],
        bump,
        seeds::program = transfer_hook_program.key()
    )]
    pub vtoken_config: Account<'info, VtokenConfig>,
    pub transfer_hook_program: Program<'info, TransferHook>,
    pub token_2022_program: Program<'info, Token2022>,
}
