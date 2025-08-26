#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod constants;
pub mod controllers;
mod instructions;
pub mod state;

use instructions::*;
use state::InvestorType;

declare_id!("3NVg4ib33KsgP8RUmCmgrbsKJnvtuhUQp2JN92NMbTiB");

#[program]
pub mod vaults {
    use super::*;

    pub fn initialize_starke(ctx: Context<InitializeStarke>) -> Result<()> {
        _initialize_starke(ctx)
    }

    pub fn pause_starke(ctx: Context<PauseOrResumeStarke>) -> Result<()> {
        _pause_starke(ctx)
    }

    pub fn resume_starke(ctx: Context<PauseOrResumeStarke>) -> Result<()> {
        _resume_starke(ctx)
    }

    pub fn add_token(
        ctx: Context<ModifyTokenWhitelist>,
        token: Pubkey,
        price_feed_id: String,
        price_update: Pubkey,
    ) -> Result<()> {
        _add_token(ctx, token, price_feed_id, price_update)
    }

    pub fn remove_token(ctx: Context<ModifyTokenWhitelist>, token: Pubkey) -> Result<()> {
        _remove_token(ctx, token)
    }

    pub fn add_manager(ctx: Context<ModifyManagerWhitelist>, manager: Pubkey) -> Result<()> {
        _add_manager(ctx, manager)
    }

    pub fn remove_manager(ctx: Context<ModifyManagerWhitelist>, manager: Pubkey) -> Result<()> {
        _remove_manager(ctx, manager)
    }

    pub fn create_vault(
        ctx: Context<CreateVault>,
        name: String,
        symbol: String,
        uri: String,
        vtoken_is_transferrable: bool,
        max_allowed_aum: Option<u64>,
        allow_retail: bool,
        allow_accredited: bool,
        allow_institutional: bool,
        allow_qualified: bool,
        individual_min_deposit: u32,
        institutional_min_deposit: u32,
        max_depositors: u32,
    ) -> Result<()> {
        _create_vault(
            ctx,
            name,
            symbol,
            uri,
            vtoken_is_transferrable,
            max_allowed_aum,
            allow_retail,
            allow_accredited,
            allow_institutional,
            allow_qualified,
            individual_min_deposit,
            institutional_min_deposit,
            max_depositors,
        )
    }

    pub fn update_vault_fees(
        ctx: Context<UpdateVaultFees>,
        new_entry_fee: u16,
        new_exit_fee: u16,
    ) -> Result<()> {
        _update_vault_fees(ctx, new_entry_fee, new_exit_fee)
    }

    pub fn deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, Deposit<'info>>,
        amount: u64,
    ) -> Result<()> {
        _deposit(ctx, amount)
    }

    pub fn withdraw<'info>(
        ctx: Context<'_, '_, 'info, 'info, Withdraw<'info>>,
        amount: u64,
    ) -> Result<()> {
        _withdraw(ctx, amount)
    }

    pub fn swap_on_jupiter(ctx: Context<SwapOnJupiter>, data: Vec<u8>) -> Result<()> {
        _swap_on_jupiter(ctx, data)
    }

    pub fn add_user(
        ctx: Context<ModifyUserWhitelist>,
        user: Pubkey,
        investor_type: InvestorType,
    ) -> Result<()> {
        _add_user(ctx, user, investor_type)
    }

    pub fn remove_user(ctx: Context<ModifyUserWhitelist>, user: Pubkey) -> Result<()> {
        _remove_user(ctx, user)
    }
}

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "Starke Finance",
    project_url: "https://starke.finance",
    contacts: "email:contact@starke.finance,discord:https://discord.gg/Kwvx8hcZBx",
    policy: "https://github.com/starke-labs/vaults/blob/main/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/starke-labs/vaults"
}
