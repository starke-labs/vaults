use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::STARKE_AUTHORITY;
use crate::controllers::{burn_vtoken, withdraw_all_tokens};
use crate::state::{
    StarkeConfig, StarkeConfigError, TokenWhitelist, Vault, VaultError, WhitelistError, Withdrawn,
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
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    let signer_seeds = &[&vault_seeds[..]];

    // Burn vtokens from depositor
    burn_vtoken(
        &ctx.accounts.user,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.user_vtoken_account,
        amount,
        signer_seeds,
        &ctx.accounts.vtoken_program,
    )?;
    msg!("{} vtokens burned successfully", amount);

    // Process withdrawals for all tokens in the vault
    withdraw_all_tokens(
        &ctx.remaining_accounts,
        &ctx.accounts.user,
        &ctx.accounts.vault,
        &ctx.accounts.vtoken_mint,
        amount,
        &ctx.accounts.whitelist,
        signer_seeds,
        &ctx.accounts.associated_token_program,
        &ctx.accounts.system_program,
    )?;

    msg!("Withdrawal completed successfully");

    emit!(Withdrawn {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        vtoken_burned_amount: amount,
        // TODO: Check if this is correct
        new_vtoken_supply: ctx.accounts.vtoken_mint.supply - amount,
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
        address = STARKE_AUTHORITY @ WhitelistError::UnauthorizedAccess,
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
    )]
    pub user_vtoken_account: Box<InterfaceAccount<'info, TokenAccount>>,

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
