use anchor_lang::prelude::*;

pub mod constants;
pub mod controllers;
mod instructions;
pub mod state;

use instructions::*;

declare_id!("STRK2VEGPAKstk6S6k5Cpin6uGtSDQkvanTaXUeaTNj");

#[program]
pub mod vaults {
    use super::*;

    pub fn initialize_whitelist(ctx: Context<InitializeWhitelist>) -> Result<()> {
        _initialize_whitelist(ctx)
    }

    pub fn add_token(ctx: Context<ModifyWhitelist>, token: Pubkey) -> Result<()> {
        _add_token(ctx, token)
    }

    pub fn create_vault(
        ctx: Context<CreateVault>,
        name: String,
        entry_fee: u16,
        exit_fee: u16,
    ) -> Result<()> {
        _create_vault(ctx, name, entry_fee, exit_fee)
    }

    pub fn update_vault_fees(
        ctx: Context<UpdateVaultFees>,
        new_entry_fee: u16,
        new_exit_fee: u16,
    ) -> Result<()> {
        _update_vault_fees(ctx, new_entry_fee, new_exit_fee)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        _deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        _withdraw(ctx, amount)
    }

    pub fn swap_on_jupiter(ctx: Context<SwapOnJupiter>, amount: u64) -> Result<()> {
        _swap_on_jupiter(ctx, amount)
    }
}

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "Starke Finance",
    project_url: "https://starke.finance",
    contacts: "email:contact@starkevalidator.com,discord:https://discord.gg/Kwvx8hcZBx",
    policy: "https://github.com/starke-labs/vaults/blob/main/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/starke-labs/vaults"
}
