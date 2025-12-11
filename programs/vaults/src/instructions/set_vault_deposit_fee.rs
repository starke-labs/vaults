use anchor_lang::prelude::*;

use crate::constants::STARKE_AUTHORITY;
use crate::state::{
    StarkeConfig, StarkeConfigError, Vault, VaultDepositFeeConfig, VaultDepositFeeConfigError,
};

pub fn _set_vault_deposit_fee(
    ctx: Context<SetVaultDepositFee>,
    enabled: bool,
    fee_rate: u16,
    platform_fee_recipient: Pubkey,
) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.authority.key(),
        STARKE_AUTHORITY,
        StarkeConfigError::Unauthorized
    );

    if ctx.accounts.deposit_fee_config.vault == Pubkey::default() {

        ctx.accounts.deposit_fee_config.initialize(
            ctx.accounts.vault.key(),
            enabled,
            fee_rate,
            platform_fee_recipient,
            ctx.bumps.deposit_fee_config,
        )?;
    } else {
       
        require!(
            ctx.accounts.deposit_fee_config.vault == ctx.accounts.vault.key(),
            VaultDepositFeeConfigError::VaultMismatch
        );
        ctx.accounts.deposit_fee_config.update(enabled, fee_rate, platform_fee_recipient)?;
    }

    msg!(
        "Deposit fee for vault {} set: enabled={}, fee_rate={} basis points, recipient={}",
        ctx.accounts.vault.key(),
        enabled,
        fee_rate,
        platform_fee_recipient
    );

    Ok(())
}

pub fn _enable_vault_deposit_fee(ctx: Context<EnableOrDisableVaultDepositFee>) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.authority.key(),
        STARKE_AUTHORITY,
        StarkeConfigError::Unauthorized
    );

    require!(
        ctx.accounts.deposit_fee_config.vault == ctx.accounts.vault.key(),
        VaultDepositFeeConfigError::VaultMismatch
    );

    ctx.accounts.deposit_fee_config.enable()?;

    msg!("Deposit fee enabled for vault {}", ctx.accounts.vault.key());

    Ok(())
}

pub fn _disable_vault_deposit_fee(ctx: Context<EnableOrDisableVaultDepositFee>) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.authority.key(),
        STARKE_AUTHORITY,
        StarkeConfigError::Unauthorized
    );

    require!(
        ctx.accounts.deposit_fee_config.vault == ctx.accounts.vault.key(),
        VaultDepositFeeConfigError::VaultMismatch
    );

    ctx.accounts.deposit_fee_config.disable()?;

    msg!("Deposit fee disabled for vault {}", ctx.accounts.vault.key());

    Ok(())
}

#[derive(Accounts)]
pub struct SetVaultDepositFee<'info> {
    #[account(mut, address = STARKE_AUTHORITY @ StarkeConfigError::Unauthorized)]
    pub authority: Signer<'info>,

    pub manager: UncheckedAccount<'info>,

    #[account(
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        init_if_needed,
        payer = authority,
        space = VaultDepositFeeConfig::MAX_SPACE,
        seeds = [VaultDepositFeeConfig::SEED, vault.key().as_ref()],
        bump,
    )]
    pub deposit_fee_config: Box<Account<'info, VaultDepositFeeConfig>>,

    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EnableOrDisableVaultDepositFee<'info> {
    #[account(mut, address = STARKE_AUTHORITY @ StarkeConfigError::Unauthorized)]
    pub authority: Signer<'info>,

    pub manager: UncheckedAccount<'info>,

    #[account(
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        seeds = [VaultDepositFeeConfig::SEED, vault.key().as_ref()],
        bump = deposit_fee_config.bump,
    )]
    pub deposit_fee_config: Box<Account<'info, VaultDepositFeeConfig>>,

    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,
}

