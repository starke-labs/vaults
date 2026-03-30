use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::STARKE_AUTHORITY;
use crate::controllers::{burn_vtoken, transfer_token_with_signer};
use crate::state::{
    StarkeConfig, StarkeConfigError, TokenWhitelist, TokenWhitelistError, Vault, VaultError,
    WithdrawnInDepositToken,
};

pub fn _withdraw_in_deposit_token<'info>(
    ctx: Context<'_, '_, 'info, 'info, WithdrawInDepositToken<'info>>,
    vtoken_amount: u64,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    // Withdraw amount should be greater than 0
    require!(vtoken_amount > 0, VaultError::InvalidAmount);

    msg!(
        "Processing withdrawal in deposit token for {} vtokens | User: {}, Vault: {}, Vtoken mint: {}",
        vtoken_amount,
        ctx.accounts.user.key(),
        ctx.accounts.vault.key(),
        ctx.accounts.vtoken_mint.key()
    );

    let (total_aum, deposit_price) = match ctx.accounts.vault.get_aum_with_deposit(
        ctx.remaining_accounts,
        &ctx.accounts.token_whitelist,
        &ctx.accounts.vault.key(),
    ) {
        Ok((aum, Some(price))) => (aum, price),
        Ok((_, None)) => return err!(VaultError::InsufficientFunds),
        Err(e) => return Err(e),
    };

    let vtoken_supply = ctx.accounts.vtoken_mint.supply;
    require!(vtoken_supply > 0, VaultError::DepositTokenSupplyZero);

    // vtoken_price_usd = aum_usd / vtoken_supply
    // Therefore, withdraw_amt_in_usd = vtoken_amount_to_withdraw * vtoken_price_usd
    let withdrawal_value = (vtoken_amount as u128)
        .checked_mul(total_aum as u128)
        .ok_or(VaultError::NumericOverflow)?
        .checked_div(vtoken_supply as u128)
        .ok_or(VaultError::NumericOverflow)?;

    // deposit_token_amount = USD value to be withdrawn / USD price of deposit token
    let transfer_deposit_token_amount = withdrawal_value
        .checked_div(deposit_price as u128)
        .ok_or(VaultError::NumericOverflow)? as u64;

    require!(transfer_deposit_token_amount > 0, VaultError::InvalidAmount);
    require!(
        ctx.accounts.vault_deposit_token_account.amount >= transfer_deposit_token_amount,
        VaultError::InsufficientFunds
    );

    let manager = ctx.accounts.manager.key();
    let signer_seeds: &[&[&[u8]]] = &[&[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]]];

    let user_balance_before = ctx.accounts.user_vtoken_account.amount;
    let will_be_zero_balance = user_balance_before == vtoken_amount;

    // Burn vtokens
    burn_vtoken(
        &ctx.accounts.user,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.user_vtoken_account,
        vtoken_amount,
        signer_seeds,
        &ctx.accounts.token_2022_program,
    )?;

    msg!("{} vtokens burned successfully", vtoken_amount);

    // Transfer deposit tokens
    transfer_token_with_signer(
        &ctx.accounts.vault_deposit_token_account,
        &ctx.accounts.user_deposit_token_account,
        transfer_deposit_token_amount,
        &ctx.accounts.deposit_token_mint,
        &ctx.accounts.vault.to_account_info(),
        signer_seeds,
        &ctx.accounts.token_program,
    )?;

    // Decrement depositor count AFTER vault-authority CPIs to avoid overwrite
    if will_be_zero_balance {
        ctx.accounts.vault.decrement_depositor_count()?;
        msg!(
            "Depositor removed. Total depositors: {}",
            ctx.accounts.vault.current_depositors
        );
    }

    msg!(
        "Withdrawal completed. User received {} deposit tokens",
        transfer_deposit_token_amount
    );

    emit!(WithdrawnInDepositToken {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        vtoken_burned_amount: vtoken_amount,
        new_vtoken_supply: vtoken_supply
            .checked_sub(vtoken_amount)
            .ok_or(VaultError::NumericOverflow)?,
        timestamp: ctx.accounts.clock.unix_timestamp,
        deposit_token_mint: ctx.accounts.deposit_token_mint.key(),
    });

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawInDepositToken<'info> {
    // Depositor
    #[account(mut)]
    pub user: Signer<'info>,

    // Program authority
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

    // Deposit token mint
    #[account(
        constraint = deposit_token_mint.key() == vault.deposit_token_mint,
    )]
    pub deposit_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        associated_token::authority = user,
        associated_token::mint = deposit_token_mint,
        payer = user,
    )]
    pub user_deposit_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::authority = vault,
        associated_token::mint = deposit_token_mint,
    )]
    pub vault_deposit_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // Starke config
    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
