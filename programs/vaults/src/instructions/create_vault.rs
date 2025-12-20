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
        ManagerWhitelist, ManagerWhitelistError, StarkeConfig, StarkeConfigError, TokenWhitelist,
        TokenWhitelistError, Vault, VaultCreated,
    },
};

#[allow(clippy::too_many_arguments)]
pub fn _create_vault(
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
    initial_vtoken_price: u32,
    management_fee_rate: u16,
    individual_max_deposit: u32,
    institutional_max_deposit: u32,
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

    if individual_min_deposit > 0 {
        msg!(
            "Individual minimum deposit amount: {}",
            individual_min_deposit
        );
    }
    if institutional_min_deposit > 0 {
        msg!(
            "Institutional minimum deposit amount: {}",
            institutional_min_deposit
        );
    }
    if max_depositors > 0 {
        msg!("Maximum depositors: {}", max_depositors);
    }

    if let Some(max_aum) = max_allowed_aum {
        msg!("Maximum allowed AUM: {}", max_aum);
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
        ctx.accounts.manager.key(),
        ctx.accounts.deposit_token_mint.key(),
        name,
        ctx.bumps.vault,
        ctx.accounts.vtoken_mint.key(),
        ctx.bumps.vtoken_mint,
        max_allowed_aum,
        allow_retail,
        allow_accredited,
        allow_institutional,
        allow_qualified,
        individual_min_deposit,
        institutional_min_deposit,
        max_depositors,
        initial_vtoken_price,
        management_fee_rate,
        individual_max_deposit,
        institutional_max_deposit,
    )?;

    msg!("Successfully created vault: {}", ctx.accounts.vault.key());

    emit!(VaultCreated {
        vault: ctx.accounts.vault.key(),
        manager: *ctx.accounts.manager.key,
        deposit_token_mint: ctx.accounts.deposit_token_mint.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        name: ctx.accounts.vault.name.to_string(),
        timestamp: ctx.accounts.clock.unix_timestamp,
        max_allowed_aum: ctx.accounts.vault.max_allowed_aum,
        initial_vtoken_price,
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
