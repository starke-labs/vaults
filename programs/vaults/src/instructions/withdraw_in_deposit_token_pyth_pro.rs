use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::STARKE_AUTHORITY;
use crate::controllers::{
    burn_vtoken, compute_token_value_usd, parse_vault_balances_for_pyth_pro,
    transfer_token_with_signer, transform_oracle_price_to_aum_decimals, verify_pyth_pro_message,
    PythProVerificationAccounts,
};
use crate::state::{
    StarkeConfig, StarkeConfigError, TokenOracleConfig, TokenWhitelist, TokenWhitelistError, Vault,
    VaultError, WithdrawnInDepositToken,
};

pub fn _withdraw_in_deposit_token_pyth_pro<'info>(
    ctx: Context<'_, '_, 'info, 'info, WithdrawInDepositTokenPythPro<'info>>,
    vtoken_amount: u64,
    price_message: Vec<u8>,
    ed25519_instruction_index: u16,
    signature_index: u8,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );
    require!(
        !ctx.accounts.vault.is_withdraw_paused(),
        VaultError::WithdrawsPaused
    );
    require!(vtoken_amount > 0, VaultError::InvalidAmount);

    let price_map = verify_pyth_pro_message(
        PythProVerificationAccounts {
            payer: &ctx.accounts.user,
            storage: &ctx.accounts.pyth_lazer_storage,
            treasury: &ctx.accounts.pyth_lazer_treasury,
            system_program: &ctx.accounts.system_program,
            instructions_sysvar: &ctx.accounts.instructions,
            program: &ctx.accounts.pyth_lazer_program,
        },
        price_message,
        ed25519_instruction_index,
        signature_index,
    )?;

    ctx.accounts
        .deposit_token_oracle_config
        .verify_pyth_pro(&ctx.accounts.deposit_token_mint.key())?;

    let vault_balances = parse_vault_balances_for_pyth_pro(
        ctx.remaining_accounts,
        &ctx.accounts.token_whitelist,
        &ctx.accounts.vault.key(),
    )?;
    let mut deposit_price = None;
    let total_aum = vault_balances
        .iter()
        .map(|b| {
            let token_price = price_map.get_checked_price(&b.oracle_config, &b.token_mint)?;
            let price_in_aum_decimals = transform_oracle_price_to_aum_decimals(&token_price)?;
            if b.token_mint == ctx.accounts.vault.deposit_token_mint {
                deposit_price =
                    compute_token_value_usd(1, b.token_decimals, price_in_aum_decimals).ok();
            }
            compute_token_value_usd(b.token_balance, b.token_decimals, price_in_aum_decimals)
        })
        .sum::<Result<u64>>()?;

    let deposit_price = deposit_price.ok_or(VaultError::UserTokenAccountNotFound)?;
    let vtoken_supply = ctx.accounts.vtoken_mint.supply;
    require!(vtoken_supply > 0, VaultError::DepositTokenSupplyZero);

    let withdrawal_value = (vtoken_amount as u128)
        .checked_mul(total_aum as u128)
        .ok_or(VaultError::NumericOverflow)?
        .checked_div(vtoken_supply as u128)
        .ok_or(VaultError::NumericOverflow)?;
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

    burn_vtoken(
        &ctx.accounts.user,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.user_vtoken_account,
        vtoken_amount,
        signer_seeds,
        &ctx.accounts.token_2022_program,
    )?;

    transfer_token_with_signer(
        &ctx.accounts.vault_deposit_token_account,
        &ctx.accounts.user_deposit_token_account,
        transfer_deposit_token_amount,
        &ctx.accounts.deposit_token_mint,
        &ctx.accounts.vault.to_account_info(),
        signer_seeds,
        &ctx.accounts.token_program,
    )?;

    if will_be_zero_balance {
        ctx.accounts.vault.decrement_depositor_count()?;
    }

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
pub struct WithdrawInDepositTokenPythPro<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        address = STARKE_AUTHORITY @ TokenWhitelistError::UnauthorizedAccess,
    )]
    pub authority: Signer<'info>,

    /// CHECK: This account is only used as the vault PDA seed.
    pub manager: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = vtoken_mint,
        associated_token::token_program = token_2022_program,
    )]
    pub user_vtoken_account: Box<InterfaceAccount<'info, TokenAccount>>,

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
    )]
    pub vtoken_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = token_whitelist.bump,
    )]
    pub token_whitelist: Box<Account<'info, TokenWhitelist>>,

    #[account(
        constraint = deposit_token_mint.key() == vault.deposit_token_mint,
    )]
    pub deposit_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [TokenOracleConfig::SEED, deposit_token_mint.key().as_ref()],
        bump = deposit_token_oracle_config.bump,
    )]
    pub deposit_token_oracle_config: Box<Account<'info, TokenOracleConfig>>,

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

    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    pub pyth_lazer_program:
        Program<'info, pyth_lazer_solana_contract::program::PythLazerSolanaContract>,
    #[account(
        seeds = [pyth_lazer_solana_contract::STORAGE_SEED],
        bump,
        seeds::program = pyth_lazer_solana_contract::ID,
    )]
    pub pyth_lazer_storage: Account<'info, pyth_lazer_solana_contract::Storage>,
    /// CHECK: checked by the Pyth Lazer verifier account constraints.
    #[account(mut)]
    pub pyth_lazer_treasury: AccountInfo<'info>,
    /// CHECK: address constrained to the standard instructions sysvar.
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
