use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::constants::STARKE_AUTHORITY;
use crate::controllers::{
    calculate_deposit_token_value, calculate_vtokens_to_mint, mint_vtoken, transfer_token,
};
use crate::state::{
    Deposited, StarkeConfig, StarkeConfigError, TokenWhitelist, Vault, VaultError, WhitelistError,
};

pub fn _deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, Deposit<'info>>,
    amount: u64,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    msg!("Processing deposit request of {} tokens", amount);
    msg!("User: {}", ctx.accounts.user.key());
    msg!("Vault: {}", ctx.accounts.vault.key());
    msg!(
        "Deposit token mint: {}",
        ctx.accounts.deposit_token_mint.key()
    );

    // Amount should be greater than 0
    require!(amount > 0, VaultError::InvalidAmount);

    // Calculate the total NAV using vault's get_nav function
    let total_nav = ctx.accounts.vault.get_nav(
        ctx.remaining_accounts,
        &ctx.accounts.whitelist,
        &ctx.accounts.vault.key(),
    )?;
    msg!("Vault NAV: {}", total_nav);

    // Calculate the USD value of deposit tokens
    let deposit_value = calculate_deposit_token_value(
        &ctx.accounts.whitelist,
        ctx.accounts.deposit_token_mint.key(),
        ctx.accounts.deposit_token_mint.decimals,
        amount,
        &ctx.accounts.deposit_token_price_update,
    )?;
    msg!("Deposit value: {}", deposit_value);

    // Calculate vtokens to mint based on NAV
    let vtokens_to_mint =
        calculate_vtokens_to_mint(total_nav, deposit_value, ctx.accounts.vtoken_mint.supply)?;
    msg!("Vtokens to mint: {}", vtokens_to_mint);

    // Transfer deposit tokens from depositor to vault
    transfer_token(
        &ctx.accounts.user_deposit_token_account,
        &ctx.accounts.vault_deposit_token_account,
        amount,
        &ctx.accounts.deposit_token_mint,
        &ctx.accounts.user,
        &ctx.accounts.token_program,
    )?;
    msg!(
        "{} tokens transferred from user to vault successfully",
        amount
    );

    // Mint vtokens to depositor
    let manager = ctx.accounts.manager.key();
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    let signer_seeds = &[&vault_seeds[..]];

    mint_vtoken(
        &ctx.accounts.vault,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.vtoken_account,
        vtokens_to_mint,
        signer_seeds,
        &ctx.accounts.vtoken_program,
    )?;
    msg!("{} vtokens minted to user successfully", vtokens_to_mint);

    msg!("Deposit completed successfully");

    emit!(Deposited {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        deposit_amount: amount,
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        vtoken_minted_amount: vtokens_to_mint,
        // TODO: Check if this is correct
        new_vtoken_supply: ctx.accounts.vtoken_mint.supply + vtokens_to_mint,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    // Depositor
    #[account(mut)]
    pub user: Signer<'info>,

    // Program authority
    // NOTE: It is necessary for the authority to be a signer as well because
    //       the remaining accounts needs to be verified
    #[account(
        mut,
        address = STARKE_AUTHORITY @ WhitelistError::UnauthorizedAccess,
    )]
    pub authority: Signer<'info>,

    // Manager
    /// CHECK: We can skip checking the manager
    pub manager: UncheckedAccount<'info>,

    // Depositor's deposit token account (from account)
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = deposit_token_mint,
    )]
    pub user_deposit_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // Vault's deposit token account (to account)
    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = vault,
        associated_token::mint = deposit_token_mint,
    )]
    pub vault_deposit_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // Depositor's vtoken account
    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = user,
        associated_token::mint = vtoken_mint,
    )]
    pub vtoken_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // Vault
    #[account(
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    // Vtoken mint
    #[account(
        mut,
        seeds = [Vault::VTOKEN_MINT_SEED, vault.key().as_ref()],
        bump = vault.mint_bump,
    )]
    pub vtoken_mint: Box<InterfaceAccount<'info, Mint>>,
    // Token program (not Token2022 yet, but might need it for using the extensions)
    pub vtoken_program: Program<'info, Token>,

    // Deposit token mint
    #[account(
        constraint = deposit_token_mint.key() == vault.deposit_token_mint,
    )]
    pub deposit_token_mint: Box<InterfaceAccount<'info, Mint>>,
    // NOTE: Changing this name to `deposit_token_program` will break the constraint for `vault_deposit_token_account`
    pub token_program: Interface<'info, TokenInterface>,

    // Deposit token price update
    pub deposit_token_price_update: Box<Account<'info, PriceUpdateV2>>,

    // Token whitelist
    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Box<Account<'info, TokenWhitelist>>,

    // Starke config
    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    pub clock: Sysvar<'info, Clock>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
