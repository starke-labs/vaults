use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::STARKE_AUTHORITY;
use crate::controllers::{
    calculate_vtokens_to_mint, compute_token_value_usd, mint_vtoken,
    parse_vault_balances_for_pyth_pro, transfer_token, transform_oracle_price_to_aum_decimals,
    verify_pyth_pro_message, PythProVerificationAccounts,
};
use crate::state::{
    Deposited, StarkeConfig, StarkeConfigError, TokenOracleConfig, TokenWhitelist,
    TokenWhitelistError, UserWhitelist, UserWhitelistError, Vault, VaultError,
};

pub fn _deposit_pyth_pro<'info>(
    ctx: Context<'_, '_, 'info, 'info, DepositPythPro<'info>>,
    amount: u64,
    price_message: Vec<u8>,
    ed25519_instruction_index: u16,
    signature_index: u8,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );
    require!(
        !ctx.accounts.vault.is_deposit_paused(),
        VaultError::DepositsPaused
    );

    msg!("Processing Pyth Pro deposit request of {} tokens", amount);

    let (investor_type, investor_tier) = ctx
        .accounts
        .user_whitelist
        .get_user_classification(&ctx.accounts.user.key())
        .ok_or(UserWhitelistError::UserNotWhitelisted)?;
    ctx.accounts
        .vault
        .validate_investor_type(investor_type, investor_tier)?;
    ctx.accounts
        .vault
        .validate_deposit_amount_by_type(amount, investor_type)?;

    let is_new_depositor = ctx.accounts.vtoken_account.amount == 0;
    ctx.accounts
        .vault
        .validate_max_depositors(is_new_depositor)?;

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
    let total_aum = vault_balances
        .iter()
        .map(|b| {
            let token_price = price_map.get_checked_price(&b.oracle_config, &b.token_mint)?;
            let price_in_aum_decimals = transform_oracle_price_to_aum_decimals(&token_price)?;
            compute_token_value_usd(b.token_balance, b.token_decimals, price_in_aum_decimals)
        })
        .sum::<Result<u64>>()?;
    msg!("Vault AUM: {}", total_aum);

    let deposit_price = price_map.get_checked_price(
        &ctx.accounts.deposit_token_oracle_config,
        &ctx.accounts.deposit_token_mint.key(),
    )?;
    let deposit_price_in_aum_decimals = transform_oracle_price_to_aum_decimals(&deposit_price)?;
    let deposit_value = compute_token_value_usd(
        amount,
        ctx.accounts.deposit_token_mint.decimals,
        deposit_price_in_aum_decimals,
    )?;
    msg!("Deposit value: {}", deposit_value);

    ctx.accounts
        .vault
        .validate_max_aum(total_aum, deposit_value)?;

    let vtokens_to_mint = calculate_vtokens_to_mint(
        total_aum,
        deposit_value,
        ctx.accounts.vtoken_mint.supply,
        ctx.accounts.vault.initial_vtoken_price,
    )?;
    msg!("Vtokens to mint: {}", vtokens_to_mint);

    transfer_token(
        &ctx.accounts.user_deposit_token_account,
        &ctx.accounts.vault_deposit_token_account,
        amount,
        &ctx.accounts.deposit_token_mint,
        &ctx.accounts.user,
        &ctx.accounts.token_program,
    )?;

    let manager = ctx.accounts.manager.key();
    let signer_seeds: &[&[&[u8]]] = &[&[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]]];
    mint_vtoken(
        &ctx.accounts.vault,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.vtoken_account,
        vtokens_to_mint,
        signer_seeds,
        &ctx.accounts.token_2022_program,
    )?;

    if is_new_depositor {
        ctx.accounts.vault.increment_depositor_count()?;
    }

    emit!(Deposited {
        vault: ctx.accounts.vault.key(),
        user: ctx.accounts.user.key(),
        deposit_amount: amount,
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        vtoken_minted_amount: vtokens_to_mint,
        new_vtoken_supply: ctx
            .accounts
            .vtoken_mint
            .supply
            .checked_add(vtokens_to_mint)
            .ok_or(VaultError::NumericOverflow)?,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct DepositPythPro<'info> {
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
        associated_token::mint = deposit_token_mint,
    )]
    pub user_deposit_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = vault,
        associated_token::mint = deposit_token_mint,
    )]
    pub vault_deposit_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::authority = user,
        associated_token::mint = vtoken_mint,
        associated_token::token_program = token_2022_program,
    )]
    pub vtoken_account: Box<InterfaceAccount<'info, TokenAccount>>,

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
        constraint = deposit_token_mint.key() == vault.deposit_token_mint,
    )]
    pub deposit_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        seeds = [TokenOracleConfig::SEED, deposit_token_mint.key().as_ref()],
        bump = deposit_token_oracle_config.bump,
    )]
    pub deposit_token_oracle_config: Box<Account<'info, TokenOracleConfig>>,

    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = token_whitelist.bump,
    )]
    pub token_whitelist: Box<Account<'info, TokenWhitelist>>,

    #[account(
        seeds = [UserWhitelist::SEED],
        bump = user_whitelist.bump,
    )]
    pub user_whitelist: Box<Account<'info, UserWhitelist>>,

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
