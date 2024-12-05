use anchor_lang::prelude::*;
use anchor_spl::token::*;

use crate::state::*;

pub fn _create_vault(ctx: Context<CreateVault>, name: String) -> Result<()> {
    ctx.accounts.vault.initialize(
        *ctx.accounts.manager.key,
        ctx.accounts.deposit_token_mint.key(),
        name,
        ctx.bumps.vault,
        ctx.accounts.vault_token_mint.key(),
        ctx.bumps.vault_token_mint,
    )?;

    emit!(VaultCreated {
        vault: ctx.accounts.vault.key(),
        manager: *ctx.accounts.manager.key,
        deposit_token: ctx.accounts.deposit_token_mint.key(),
        vault_token_mint: ctx.accounts.vault_token_mint.key(),
        name: ctx.accounts.vault.name.to_string(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    // Vault manager
    #[account(mut)]
    pub manager: Signer<'info>,

    // Vault (PDA)
    // Currently seeded by manager pubkey, which means that the manager can only create one vault
    // This also means when users want to deposit, they need to send manager's pubkey in accounts
    // Do we want this? Or should we avoid PDA here and store vault pubkeys instead?
    // TODO: Find a way to allow multiple vaults per manager for v1
    #[account(
        init,
        payer = manager,
        space = Vault::MAX_SPACE,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,

    // Vault token mint
    #[account(
        init,
        payer = manager,
        seeds = [Vault::VAULT_TOKEN_MINT_SEED, vault.key().as_ref()],
        bump,
        mint::decimals = deposit_token_mint.decimals,
        mint::authority = vault,
        mint::freeze_authority = vault,
    )]
    pub vault_token_mint: Account<'info, Mint>,

    // Whitelist
    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Account<'info, TokenWhitelist>,

    // Deposit token mint
    #[account(
        constraint = whitelist.is_whitelisted(&deposit_token_mint.key()) @ WhitelistError::TokenNotWhitelisted,
    )]
    pub deposit_token_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}
