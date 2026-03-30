use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::STARKE_AUTHORITY;
use crate::controllers::{burn_vtoken, withdraw_all_tokens};
use crate::state::{
    StarkeConfig, StarkeConfigError, TokenWhitelist, TokenWhitelistError, Vault, VaultError,
    Withdrawn,
};

pub fn _withdraw<'info>(
    ctx: Context<'_, '_, 'info, 'info, Withdraw<'info>>,
    amount: u64,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    msg!("Processing withdrawal request of {} vtokens", amount);
    msg!("User: {}", ctx.accounts.user.key());
    msg!("Vault: {}", ctx.accounts.vault.key());
    msg!("Vtoken mint: {}", ctx.accounts.vtoken_mint.key());

    // Amount should be greater than 0
    require!(amount > 0, VaultError::InvalidAmount);

    let manager = ctx.accounts.manager.key();
    let signer_seeds: &[&[&[u8]]] = &[&[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]]];

    // Check if user will have zero vtokens after withdrawal (becoming a non-depositor)
    let user_balance_before = ctx.accounts.user_vtoken_account.amount;
    let will_be_zero_balance = user_balance_before == amount;

    // Burn vtokens from depositor
    burn_vtoken(
        &ctx.accounts.user,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.user_vtoken_account,
        amount,
        signer_seeds,
        &ctx.accounts.token_2022_program,
    )?;
    msg!("{} vtokens burned successfully", amount);

    // Process withdrawals for all tokens in the vault
    withdraw_all_tokens(
        ctx.remaining_accounts,
        &ctx.accounts.user,
        &ctx.accounts.vault,
        &ctx.accounts.vtoken_mint,
        amount,
        &ctx.accounts.token_whitelist,
        signer_seeds,
        &ctx.accounts.associated_token_program,
        &ctx.accounts.system_program,
    )?;

    // Decrement depositor count AFTER vault-authority CPIs to avoid overwrite
    if will_be_zero_balance {
        ctx.accounts.vault.decrement_depositor_count()?;
        msg!("Depositor removed. Total depositors: {}", ctx.accounts.vault.current_depositors);
    }

    msg!("Withdrawal completed successfully");

    emit!(Withdrawn {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        vtoken_burned_amount: amount,
        // TODO: Check if this is correct
        new_vtoken_supply: ctx.accounts.vtoken_mint.supply.checked_sub(amount).unwrap(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // Depositor
    #[account(mut)]
    pub user: Signer<'info>,

    // Program authority
    // NOTE: It is necessary for the authority to be a signer as well because
    //       the remaining accounts needs to be verified
    #[account(
        mut,
        address = STARKE_AUTHORITY @ TokenWhitelistError::UnauthorizedAccess,
    )]
    pub authority: Signer<'info>,

    // Manager
    /// CHECK: We can skip checking the manager
    pub manager: UncheckedAccount<'info>,

    // Depositor's vtoken account
    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = vtoken_mint,
        associated_token::token_program = token_2022_program,
    )]
    pub user_vtoken_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // Vault
    #[account(
        mut,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    // Vtoken mint
    #[account(
        mut,
        seeds = [Vault::VTOKEN_MINT_SEED, vault.key().as_ref()],
        bump = vault.mint_bump,
        mint::token_program = token_2022_program,
    )]
    pub vtoken_mint: Box<InterfaceAccount<'info, Mint>>,

    // Token whitelist
    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = token_whitelist.bump,
    )]
    pub token_whitelist: Box<Account<'info, TokenWhitelist>>,

    // Starke config
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
