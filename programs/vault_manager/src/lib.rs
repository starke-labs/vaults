use anchor_lang::prelude::*;

pub mod constants;
pub mod controllers;
mod instructions;
pub mod state;

use instructions::*;

declare_id!("5bZpuR8pbHpndAcJ99Hc3fNYT65fagRCptFvCwqPw3Te");

#[program]
pub mod vault_manager {
    use super::*;

    pub fn initialize_whitelist(ctx: Context<InitializeWhitelist>) -> Result<()> {
        _initialize_whitelist(ctx)
    }

    pub fn add_token(ctx: Context<ModifyWhitelist>, token: Pubkey) -> Result<()> {
        _add_token(ctx, token)
    }

    pub fn create_vault(ctx: Context<CreateVault>, name: String) -> Result<()> {
        _create_vault(ctx, name)
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
