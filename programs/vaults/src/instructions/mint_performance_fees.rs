use crate::{
    controllers::{
        calculate_performance_fee_vtokens_to_mint, compute_token_price, mint_vtoken,
    },
    state::{PerformanceFeeMinted, StarkeConfig, StarkeConfigError, TokenWhitelist, Vault, VaultError},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

pub fn _mint_performance_fees(ctx: Context<MintPerformanceFees>) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    // Validate manager is the vault manager
    require_keys_eq!(
        ctx.accounts.manager.key(),
        ctx.accounts.vault.manager,
        VaultError::Unauthorized
    );

    // Validate performance fee is enabled
    require!(
        ctx.accounts.vault.performance_fees_rate > 0,
        VaultError::FeesNotDue
    );

    let current_vtoken_supply = ctx.accounts.vtoken_mint.supply;
    require!(current_vtoken_supply > 0, VaultError::NoVtokenSupply);

    // Calculate current AUM
    let current_aum = ctx.accounts.vault.get_aum(
        ctx.remaining_accounts,
        &ctx.accounts.token_whitelist,
        &ctx.accounts.vault.key(),
    )?;

    // Calculate current token price
    let current_token_price = compute_token_price(current_aum, current_vtoken_supply)?;

    // Get high-water mark (last performance fee token price)
    let last_perf_fee_token_price = ctx.accounts.vault.last_perf_fee_token_price;

    // Check if this is the first time (last_perf_fee_token_price == 0)
    // If so, set initial HWM to current price and return (no fee on first call)
    if last_perf_fee_token_price == 0 {
        // First time: set HWM to current price, no fee this time
        // This ensures we only charge fees on NEW profits above the starting point
        ctx.accounts.vault.last_perf_fee_token_price = current_token_price;
        ctx.accounts.vault.last_perf_fee_timestamp = ctx.accounts.clock.unix_timestamp;
        return Ok(());
    }

    // Check if we're above high-water mark
    if current_token_price <= last_perf_fee_token_price {
        // No profit above high-water mark
        return Err(error!(VaultError::FeesNotDue));
    }

    // Calculate performance fee vTokens to mint
    let vtokens_to_mint = calculate_performance_fee_vtokens_to_mint(
        current_token_price,
        last_perf_fee_token_price,
        current_vtoken_supply,
        ctx.accounts.vault.performance_fees_rate,
    )?;

    require!(vtokens_to_mint > 0, VaultError::NoVtokenSupply);

    // Mint vTokens to manager
    let manager = ctx.accounts.manager.key();
    let signer_seeds: &[&[&[u8]]] = &[&[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]]];

    mint_vtoken(
        &ctx.accounts.vault,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.manager_vtoken_account,
        vtokens_to_mint,
        signer_seeds,
        &ctx.accounts.token_2022_program,
    )?;

    let new_supply = current_vtoken_supply
        .checked_add(vtokens_to_mint)
        .ok_or(error!(VaultError::NumericOverflow))?;

    // Update high-water mark to current price
    ctx.accounts.vault.last_perf_fee_token_price = current_token_price;
    ctx.accounts.vault.last_perf_fee_timestamp = ctx.accounts.clock.unix_timestamp;

    emit!(PerformanceFeeMinted {
        vault: ctx.accounts.vault.key(),
        manager: ctx.accounts.manager.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        vtoken_fee_amount: vtokens_to_mint,
        new_vtoken_supply: new_supply,
        current_token_price,
        high_water_mark: current_token_price, // Updated HWM
        previous_high_water_mark: last_perf_fee_token_price,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct MintPerformanceFees<'info> {
    // Manager
    #[account(mut)]
    pub manager: Signer<'info>,

    // Vault
    #[account(
        mut,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        seeds = [Vault::VTOKEN_MINT_SEED, vault.key().as_ref()],
        bump = vault.mint_bump,
        mint::token_program = token_2022_program,
        constraint = vault.mint == vtoken_mint.key() @ VaultError::InvalidVtokenMint
    )]
    pub vtoken_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = manager,
        associated_token::authority = manager,
        associated_token::mint = vtoken_mint,
        associated_token::token_program = token_2022_program,
    )]
    pub manager_vtoken_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // Token whitelist (needed for AUM calculation)
    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = token_whitelist.bump,
    )]
    pub token_whitelist: Box<Account<'info, TokenWhitelist>>,

    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    pub clock: Sysvar<'info, Clock>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

