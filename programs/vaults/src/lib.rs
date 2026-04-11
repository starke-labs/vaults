#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod constants;
pub mod controllers;
mod instructions;
pub mod state;

use instructions::*;
use state::{InvestorTier, InvestorType, InvestorTypeWithRange};

declare_id!("56gFPCzaTGNJQcZrpfewDDgGYD8SR7G2RCrxy5z26jch");

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

    #[allow(clippy::too_many_arguments)]
    pub fn create_vault(
        ctx: Context<CreateVault>,
        name: String,
        symbol: String,
        uri: String,
        vtoken_is_transferrable: bool,
        max_allowed_aum: u64,
        initial_vtoken_price: u32,
        allowed_investor_types: Vec<InvestorType>,
        allowed_investor_tiers: Vec<InvestorTier>,
        range_allowed_per_investor_type: Vec<InvestorTypeWithRange>,
        max_depositors: u32,
        management_fee_rate: u16,
    ) -> Result<()> {
        _create_vault(
            ctx,
            name,
            symbol,
            uri,
            vtoken_is_transferrable,
            max_allowed_aum,
            initial_vtoken_price,
            allowed_investor_types,
            allowed_investor_tiers,
            range_allowed_per_investor_type,
            max_depositors,
            management_fee_rate,
        )
    }

    pub fn close_vault(ctx: Context<CloseVault>) -> Result<()> {
        _close_vault(ctx)
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

    pub fn mint_management_fees(ctx: Context<MintManagementFees>) -> Result<()> {
        _mint_management_fees(ctx)
    }
    pub fn withdraw_in_deposit_token<'info>(
        ctx: Context<'_, '_, 'info, 'info, WithdrawInDepositToken<'info>>,
        amount: u64,
    ) -> Result<()> {
        _withdraw_in_deposit_token(ctx, amount)
    }

    pub fn swap_on_jupiter(ctx: Context<SwapOnJupiter>, data: Vec<u8>) -> Result<()> {
        _swap_on_jupiter(ctx, data)
    }

    pub fn swap_to_deposit_token_on_jupiter<'info>(
        ctx: Context<'_, '_, 'info, 'info, SwapToDepositTokenOnJupiter<'info>>,
        data: Vec<u8>,
        aum_accounts_len: u64,
    ) -> Result<()> {
        _swap_to_deposit_token_on_jupiter(ctx, data, aum_accounts_len)
    }

    pub fn add_user(
        ctx: Context<ModifyUserWhitelist>,
        user: Pubkey,
        investor_type: InvestorType,
        investor_tier: InvestorTier,
    ) -> Result<()> {
        _add_user(ctx, user, investor_type, investor_tier)
    }

    pub fn remove_user(ctx: Context<ModifyUserWhitelist>, user: Pubkey) -> Result<()> {
        _remove_user(ctx, user)
    }

    pub fn migrate_user_whitelist(ctx: Context<MigrateUserWhitelist>) -> Result<()> {
        _migrate_user_whitelist(ctx)
    }

    pub fn pause_deposits(ctx: Context<PauseOrResumeDeposits>) -> Result<()> {
        _pause_deposits(ctx)
    }

    pub fn resume_deposits(ctx: Context<PauseOrResumeDeposits>) -> Result<()> {
        _resume_deposits(ctx)
    }

    pub fn pause_withdraws(ctx: Context<PauseOrResumeWithdraws>) -> Result<()> {
        _pause_withdraws(ctx)
    }

    pub fn resume_withdraws(ctx: Context<PauseOrResumeWithdraws>) -> Result<()> {
        _resume_withdraws(ctx)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_fund(
        ctx: Context<UpdateFund>,
        max_allowed_aum: u64,
        allowed_investor_types: Vec<InvestorType>,
        allowed_investor_tiers: Vec<InvestorTier>,
        range_allowed_per_investor_type: Vec<InvestorTypeWithRange>,
        max_depositors: u32,
        management_fee_rate: u16,
        is_transferrable: bool,
    ) -> Result<()> {
        _update_fund(
            ctx,
            max_allowed_aum,
            allowed_investor_types,
            allowed_investor_tiers,
            range_allowed_per_investor_type,
            max_depositors,
            management_fee_rate,
            is_transferrable,
        )
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
