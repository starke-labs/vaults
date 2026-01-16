use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::STARKE_AUTHORITY;
use crate::controllers::{burn_vtoken, transfer_token_with_signer};
use crate::state::{
    StarkeConfig, StarkeConfigError, TokenWhitelist, TokenWhitelistError, UserDepositInfo, Vault, VaultError,
    WithdrawnInDepositToken,
};

pub fn _withdraw_in_deposit_token<'info>(
    ctx: Context<'_, '_, 'info, 'info, WithdrawInDepositToken<'info>>,
    amount: u64,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    // Amount should be greater than 0
    require!(amount > 0, VaultError::InvalidAmount);

    // Check if lock-in period has expired (only if UserDepositInfo exists)
    // If it doesn't exist, the user deposited before this feature was implemented, so no lock-in applies
    if !ctx.accounts.user_deposit_info.data_is_empty() {
        match UserDepositInfo::try_deserialize(&mut &ctx.accounts.user_deposit_info.data.borrow()[..]) {
            Ok(user_deposit_info) => {
                if !user_deposit_info.is_lock_in_expired(ctx.accounts.clock.unix_timestamp) {
                    let elapsed = ctx.accounts.clock.unix_timestamp
                        .checked_sub(user_deposit_info.first_deposit_timestamp)
                        .unwrap_or(0);
                    let remaining = user_deposit_info.lock_in_period_seconds as i64 - elapsed;
                    msg!(
                        "Lock-in period not expired. Remaining: {} seconds",
                        remaining
                    );
                    return Err(VaultError::LockInPeriodNotExpired.into());
                }
            }
            Err(_) => {
                // Account exists but can't be deserialized - treat as no lock-in for safety
                msg!("UserDepositInfo exists but deserialization failed - allowing withdrawal");
            }
        }
    } else {
        msg!("UserDepositInfo not found - user deposited before lock-in feature, allowing withdrawal");
    }

    msg!(
        "Processing withdrawal for {} vtokens | User: {}, Vault: {}, Vtoken mint: {}",
        amount,
        ctx.accounts.user.key(),
        ctx.accounts.vault.key(),
        ctx.accounts.vtoken_mint.key()
    );

    let (total_aum, deposit_token_value) = match ctx.accounts.vault.get_aum_with_deposit(
        ctx.remaining_accounts,
        &ctx.accounts.token_whitelist,
        &ctx.accounts.vault.key(),
    ) {
        (Ok(v), Some(d)) => (v, d),
        (Err(e), _) => return Err(e),
        (_, None) => return err!(VaultError::InsufficientFunds),
    };
    let vtoken_price = total_aum.saturating_div(ctx.accounts.vtoken_mint.supply);
    let transfer_deposit_token_amount = amount
        .checked_mul(vtoken_price)
        .ok_or(VaultError::NumericOverflow)?
        .checked_div(deposit_token_value)
        .ok_or(VaultError::NumericOverflow)?;
    // Insufficient deposit token amount to withdraw
    require!(
        amount
            .checked_mul(vtoken_price)
            .is_some_and(|v| v <= deposit_token_value),
        VaultError::InsufficientFunds
    );

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

    // Decrement depositor count if user withdrew all their tokens
    if will_be_zero_balance {
        ctx.accounts.vault.decrement_depositor_count()?;
        msg!(
            "Depositor removed. Total depositors: {}",
            ctx.accounts.vault.current_depositors
        );
    }

    transfer_token_with_signer(
        &ctx.accounts.vault_deposit_token_account,
        &ctx.accounts.user_deposit_token_account,
        transfer_deposit_token_amount,
        &ctx.accounts.deposit_token_mint,
        &ctx.accounts.vault.to_account_info(),
        signer_seeds,
        &ctx.accounts.token_program,
    )?;

    msg!("Withdrawal completed successfully");

    emit!(WithdrawnInDepositToken {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        vtoken_burned_amount: amount,
        // TODO: Check if this is correct
        new_vtoken_supply: ctx.accounts.vtoken_mint.supply.checked_sub(amount).unwrap(),
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

    // User deposit info (to check lock-in period)
    // Optional - may not exist for users who deposited before this feature was implemented
    /// CHECK: Account may not exist or be uninitialized, we check in the function
    pub user_deposit_info: UncheckedAccount<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
