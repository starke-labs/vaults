use anchor_lang::prelude::*;
use anchor_spl::{
    token::Token,
    token_interface::{Mint, Token2022},
};

use crate::controllers::*;
use crate::state::*;

pub fn _create_vault(
    ctx: Context<CreateVault>,
    name: String,
    entry_fee: u16,
    exit_fee: u16,
) -> Result<()> {
    ctx.accounts.vault.initialize(
        *ctx.accounts.manager.key,
        ctx.accounts.deposit_token_mint.key(),
        name.clone(),
        ctx.bumps.vault,
        ctx.accounts.vault_token_mint.key(),
        ctx.bumps.vault_token_mint,
        entry_fee,
        exit_fee,
    )?;

    let manager = ctx.accounts.manager.key();
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    let signer_seeds = &[&vault_seeds[..]];
    // TODO: Get name, symbol, uri from args
    let symbol = "svSTARKE";
    let uri = "https://starke.finance";
    initialize_token_metadata(
        ctx.accounts.vault_token_mint.clone(),
        ctx.accounts.vault.clone(),
        symbol.to_string(),
        uri.to_string(),
        signer_seeds,
        ctx.accounts.token_2022_program.clone(),
    )?;

    update_account_lamports_to_minimum_balance(
        ctx.accounts.vault_token_mint.to_account_info(),
        ctx.accounts.manager.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    )?;

    // TODO: Add mint pubkey, metadata pointer, symbol, uri, etc.
    emit!(VaultCreated {
        vault: ctx.accounts.vault.key(),
        manager: *ctx.accounts.manager.key,
        deposit_token: ctx.accounts.deposit_token_mint.key(),
        vault_token_mint: ctx.accounts.vault_token_mint.key(),
        name: ctx.accounts.vault.name.to_string(),
        timestamp: ctx.accounts.clock.unix_timestamp,
        entry_fee: ctx.accounts.vault.entry_fee,
        exit_fee: ctx.accounts.vault.exit_fee,
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
        mint::token_program = token_program,
        mint::decimals = deposit_token_mint.decimals,
        mint::authority = vault,
        mint::freeze_authority = vault,
        extensions::metadata_pointer::metadata_address = vault_token_mint,
        extensions::metadata_pointer::authority = vault,
    )]
    pub vault_token_mint: InterfaceAccount<'info, Mint>,

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
    pub deposit_token_mint: InterfaceAccount<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub clock: Sysvar<'info, Clock>,
}
