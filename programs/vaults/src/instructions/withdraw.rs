use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::PROGRAM_AUTHORITY;
use crate::controllers::{burn_vtoken, withdraw_all_tokens};
use crate::state::{TokenWhitelist, Vault, WhitelistError, WithdrawMade};

pub fn _withdraw<'info>(
    ctx: Context<'_, '_, 'info, 'info, Withdraw<'info>>,
    amount: u64,
) -> Result<()> {
    msg!("Withdraw instruction called with amount: {}", amount);

    let manager = ctx.accounts.manager.key();
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    let signer_seeds = &[&vault_seeds[..]];
    msg!("Vault seeds generated");

    // Burn vtokens from depositor
    msg!("Burning vtokens: {}", amount);
    burn_vtoken(
        &ctx.accounts.user,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.user_vtoken_account,
        amount,
        signer_seeds,
        &ctx.accounts.token_program,
    )?;
    msg!("Vtokens burned successfully");

    // Process withdrawals for all tokens in the vault
    msg!("Processing withdrawals for all vault tokens");
    withdraw_all_tokens(
        &ctx.remaining_accounts,
        &ctx.accounts.user,
        &ctx.accounts.vault,
        &ctx.accounts.vtoken_mint,
        amount,
        &ctx.accounts.whitelist,
        signer_seeds,
        &ctx.accounts.token_program,
        &ctx.accounts.associated_token_program,
        &ctx.accounts.system_program,
    )?;
    msg!("All token withdrawals processed successfully");

    emit!(WithdrawMade {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        amount,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    msg!("Withdraw event emitted");

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
        address = PROGRAM_AUTHORITY @ WhitelistError::UnauthorizedAccess,
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
    pub user_vtoken_account: Box<Account<'info, TokenAccount>>,

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
    pub vtoken_mint: Box<Account<'info, Mint>>,

    // Token whitelist
    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Box<Account<'info, TokenWhitelist>>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
