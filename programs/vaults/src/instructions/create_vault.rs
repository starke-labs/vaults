use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::Metadata,
    token_interface::{Mint, TokenInterface},
};

use crate::{
    constants::NAV_DECIMALS,
    controllers::initialize_token_metadata,
    state::{TokenWhitelist, Vault, VaultCreated, WhitelistError},
};

pub fn _create_vault(
    ctx: Context<CreateVault>,
    name: &str,
    symbol: &str,
    uri: &str,
    entry_fee: u16,
    exit_fee: u16,
) -> Result<()> {
    msg!("Processing vault creation request");
    msg!("Manager: {}", ctx.accounts.manager.key());
    msg!(
        "Deposit token mint: {}",
        ctx.accounts.deposit_token_mint.key()
    );
    msg!("Name: {}", name);
    msg!("Entry fee: {}, Exit fee: {}", entry_fee, exit_fee);

    // Initialize vtoken metadata
    let manager = ctx.accounts.manager.key();
    let vtoken_mint_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.bumps.vault]];
    let signer_seeds = &[&vtoken_mint_seeds[..]];

    initialize_token_metadata(
        name,
        symbol,
        uri,
        &ctx.accounts.manager,
        &ctx.accounts.metadata,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.vault.to_account_info(),
        signer_seeds,
        &ctx.accounts.rent,
        &ctx.accounts.metadata_program,
        &ctx.accounts.system_program,
    )?;
    msg!("Successfully initialized vtoken metadata");

    // Initialize vault
    ctx.accounts.vault.initialize(
        &ctx.accounts.manager.key(),
        &ctx.accounts.deposit_token_mint.key(),
        name,
        ctx.bumps.vault,
        &ctx.accounts.vtoken_mint.key(),
        ctx.bumps.vtoken_mint,
        entry_fee,
        exit_fee,
    )?;

    msg!("Successfully created vault: {}", ctx.accounts.vault.key());

    emit!(VaultCreated {
        vault: ctx.accounts.vault.key(),
        manager: *ctx.accounts.manager.key,
        deposit_token_mint: ctx.accounts.deposit_token_mint.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
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
    pub vault: Box<Account<'info, Vault>>,

    // Vtoken mint
    #[account(
        init,
        payer = manager,
        seeds = [Vault::VTOKEN_MINT_SEED, vault.key().as_ref()],
        bump,
        // TODO: Add metadata
        mint::decimals = NAV_DECIMALS,
        mint::authority = vault,
        mint::freeze_authority = vault,
    )]
    pub vtoken_mint: Box<InterfaceAccount<'info, Mint>>,

    // Vtoken metadata
    /// CHECK: This account will be initialized by Metaplex
    #[account(
        mut,
        seeds = [b"metadata", metadata_program.key().as_ref(), vtoken_mint.key().as_ref()],
        bump,
        seeds::program = metadata_program.key(),
    )]
    pub metadata: UncheckedAccount<'info>,

    // Whitelist
    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Box<Account<'info, TokenWhitelist>>,

    // Deposit token mint
    #[account(
        constraint = whitelist.is_whitelisted(&deposit_token_mint.key()) @ WhitelistError::TokenNotWhitelisted,
    )]
    pub deposit_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub metadata_program: Program<'info, Metadata>,
    pub clock: Sysvar<'info, Clock>,
}
