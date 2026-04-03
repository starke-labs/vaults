use anchor_lang::{prelude::*, solana_program};
use anchor_spl::{
    metadata::Metadata,
    token_2022::Token2022,
    token_interface::{Mint, TokenInterface},
};
use transfer_hook::{
    constants::EXTRA_ACCOUNT_METAS_SEED, program::TransferHook, state::VtokenConfig,
};

use crate::{
    constants::AUM_DECIMALS,
    controllers::{initialize_token_metadata, initialize_vtoken_config, update_token_metadata},
    state::{
        InvestorTier, InvestorType, InvestorTypeWithRange, ManagerWhitelist, ManagerWhitelistError,
        StarkeConfig, StarkeConfigError, TokenWhitelist, TokenWhitelistError, Vault, VaultCreated,
    },
};

#[allow(clippy::too_many_arguments)]
pub fn _create_vault(
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
    msg!("Creating vault with investor type restrictions");

    if max_depositors > 0 {
        msg!("Maximum depositors: {}", max_depositors);
    }

    if max_allowed_aum > 0 {
        msg!("Maximum allowed AUM: {}", max_allowed_aum);
    }

    // Vault seeds
    let manager = ctx.accounts.manager.key();
    let signer_seeds: &[&[&[u8]]] = &[&[Vault::SEED, manager.as_ref(), &[ctx.bumps.vault]]];

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

    if ctx.accounts.metadata.data_is_empty() {
        // Initialize vtoken metadata
        initialize_token_metadata(
            name.clone(),
            symbol,
            uri,
            &ctx.accounts.manager,
            &ctx.accounts.metadata,
            &ctx.accounts.vtoken_mint,
            &ctx.accounts.vault.to_account_info(),
            signer_seeds,
            &ctx.accounts.instructions,
            &ctx.accounts.token_2022_program,
            &ctx.accounts.metadata_program,
            &ctx.accounts.system_program,
        )?;
        msg!("Successfully initialized vtoken metadata");
    } else {
        update_token_metadata(
            name.clone(),
            symbol,
            uri,
            &ctx.accounts.manager,
            &ctx.accounts.metadata,
            &ctx.accounts.vtoken_mint,
            &ctx.accounts.vault.to_account_info(),
            signer_seeds,
            &ctx.accounts.instructions,
            &ctx.accounts.metadata_program,
            &ctx.accounts.system_program,
        )?;
        msg!("Updating vtoken metadata initialization");
    }

    // Initialize vault
    ctx.accounts.vault.initialize(
        ctx.accounts.manager.key(),            // manager
        ctx.accounts.deposit_token_mint.key(), // deposit_token_mint
        name.clone(),                          // name
        ctx.bumps.vault,                       // bump
        ctx.accounts.vtoken_mint.key(),        // vtoken_mint
        ctx.bumps.vtoken_mint,                 // vtoken_mint_bump
        max_allowed_aum,                       // max_allowed_aum
        initial_vtoken_price,                  // initial_vtoken_price
        allowed_investor_types
            .into_iter()
            .fold(0, |acc, x| acc | x as u16), // allowed_investor_types
        allowed_investor_tiers
            .into_iter()
            .fold(0, |acc, x| acc | x as u16), // allowed_investor_tiers
        range_allowed_per_investor_type,       // min_max_deposits_per_investor_type
        max_depositors,                        // max_depositors
        management_fee_rate,                   // management_fees_rate
    )?;

    // Create fund page in the app relies on this log message (DON'T CHANGE OR REMOVE)
    msg!("Successfully created vault: {}", ctx.accounts.vault.key());

    emit!(VaultCreated {
        vault: ctx.accounts.vault.key(),
        manager: *ctx.accounts.manager.key,
        deposit_token_mint: ctx.accounts.deposit_token_mint.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        name: ctx.accounts.vault.name.to_string(),
        timestamp: ctx.accounts.clock.unix_timestamp,
        max_allowed_aum: ctx.accounts.vault.max_allowed_aum,
        initial_vtoken_price: ctx.accounts.vault.initial_vtoken_price,
        allowed_investor_types: ctx.accounts.vault.allowed_investor_types,
        allowed_investor_tiers: ctx.accounts.vault.allowed_investor_tiers,
        range_allowed_per_investor_type: ctx.accounts.vault.range_allowed_per_investor_type.clone(),
        max_depositors: ctx.accounts.vault.max_depositors,
        management_fee_rate: ctx.accounts.vault.management_fees_rate,
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
        init_if_needed,
        payer = manager,
        seeds = [Vault::VTOKEN_MINT_SEED, vault.key().as_ref()],
        bump,
        // TODO: Add metadata
        mint::decimals = AUM_DECIMALS,
        mint::authority = vault,
        mint::freeze_authority = vault,
        mint::token_program = token_2022_program,
        extensions::transfer_hook::program_id = transfer_hook_program,
        extensions::transfer_hook::authority = manager,
    )]
    pub vtoken_mint: InterfaceAccount<'info, Mint>,

    // Vtoken metadata
    /// CHECK: This account will be initialized by Metaplex
    #[account(
        mut,
        seeds = [b"metadata", metadata_program.key().as_ref(), vtoken_mint.key().as_ref()],
        bump,
        seeds::program = metadata_program.key(),
    )]
    pub metadata: AccountInfo<'info>,

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
        mut,
        seeds = [EXTRA_ACCOUNT_METAS_SEED, vtoken_mint.key().as_ref()],
        bump,
        seeds::program = transfer_hook_program.key(),
    )]
    pub extra_account_metas: AccountInfo<'info>,

    // Vtoken config
    /// CHECK: This account will be initialized by the transfer hook program
    #[account(
        mut,
        seeds = [VtokenConfig::SEED, vtoken_mint.key().as_ref()],
        bump,
        seeds::program = transfer_hook_program.key(),
    )]
    pub vtoken_config: AccountInfo<'info>,

    // Transfer hook program
    pub transfer_hook_program: Program<'info, TransferHook>,

    // Instructions sysvar
    // TODO: Check why anchor doesn't support the instruction sysvar
    /// CHECK: create metadata account ix needs this sysvar
    #[account(address = solana_program::sysvar::instructions::ID)]
    pub instructions: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    // For deposit token mint
    pub token_program: Interface<'info, TokenInterface>,
    // For vtoken mint (Token2022)
    pub token_2022_program: Program<'info, Token2022>,
    pub metadata_program: Program<'info, Metadata>,
    pub clock: Sysvar<'info, Clock>,
}
