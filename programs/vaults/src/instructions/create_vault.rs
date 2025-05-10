use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::Metadata,
    token_interface::{Mint, TokenInterface},
};
use transfer_hook::{
    constants::EXTRA_ACCOUNT_METAS_SEED, program::TransferHook, state::VtokenConfig,
};

use crate::{
    constants::NAV_DECIMALS,
    controllers::{initialize_token_metadata, initialize_transfer_hook, initialize_vtoken_config},
    state::{
        ManagerWhitelist, ManagerWhitelistError, StarkeConfig, StarkeConfigError, TokenWhitelist,
        TokenWhitelistError, Vault, VaultCreated,
    },
};

pub fn _create_vault(
    ctx: Context<CreateVault>,
    name: &str,
    symbol: &str,
    uri: &str,
    entry_fee: u16,
    exit_fee: u16,
    vtoken_is_transferrable: bool,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    msg!("Processing vault creation request");
    msg!("Manager: {}", ctx.accounts.manager.key());
    msg!(
        "Deposit token mint: {}",
        ctx.accounts.deposit_token_mint.key()
    );
    msg!("Name: {}", name);
    msg!("Entry fee: {}, Exit fee: {}", entry_fee, exit_fee);

    // Vault seeds
    let manager = ctx.accounts.manager.key();
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.bumps.vault]];
    let signer_seeds = &[&vault_seeds[..]];

    // Initialize vtoken transfer hook
    initialize_transfer_hook(
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.transfer_hook_program,
        &ctx.accounts.vault.to_account_info(),
        &ctx.accounts.token_program,
        signer_seeds,
    )?;
    msg!("Successfully initialized vtoken transfer hook");

    // Initialize vtoken config
    initialize_vtoken_config(
        vtoken_is_transferrable,
        &ctx.accounts.manager,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.extra_account_metas,
        &ctx.accounts.vtoken_config,
        &ctx.accounts.transfer_hook_program,
        &ctx.accounts.system_program,
    )?;
    msg!("Successfully initialized vtoken config");

    // Initialize vtoken metadata
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
    #[account(
        mut,
        constraint = manager_whitelist.is_whitelisted(&manager.key()) @ ManagerWhitelistError::ManagerNotWhitelisted
    )]
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

    // Token Whitelist
    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = token_whitelist.bump,
    )]
    pub token_whitelist: Box<Account<'info, TokenWhitelist>>,

    // Manager Whitelist
    #[account(
        seeds = [ManagerWhitelist::SEED],
        bump = manager_whitelist.bump,
    )]
    pub manager_whitelist: Box<Account<'info, ManagerWhitelist>>,

    // Deposit token mint
    #[account(
        constraint = token_whitelist.is_whitelisted(&deposit_token_mint.key()) @ TokenWhitelistError::TokenNotWhitelisted,
    )]
    pub deposit_token_mint: Box<InterfaceAccount<'info, Mint>>,

    // Starke config
    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    // Extra account metas
    /// CHECK: This account is initialized in the initialize_extra_account_metas instruction
    /// and its data is structured according to spl_tlv_account_resolution::state::ExtraAccountMetaList.
    /// The Token2022 program validates this account during CPI.
    #[account(
        seeds = [EXTRA_ACCOUNT_METAS_SEED, vtoken_mint.key().as_ref()],
        bump,
        seeds::program = transfer_hook_program.key(),
    )]
    pub extra_account_metas: AccountInfo<'info>,

    // Vtoken config
    #[account(
        seeds = [VtokenConfig::SEED, vtoken_mint.key().as_ref()],
        bump,
        seeds::program = transfer_hook_program.key(),
    )]
    pub vtoken_config: Box<Account<'info, VtokenConfig>>,

    // Transfer hook program
    pub transfer_hook_program: Program<'info, TransferHook>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub metadata_program: Program<'info, Metadata>,
    pub clock: Sysvar<'info, Clock>,
}
