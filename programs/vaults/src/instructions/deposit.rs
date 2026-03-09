use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::constants::STARKE_AUTHORITY;
use crate::controllers::{
    calculate_deposit_token_value, calculate_vtokens_to_mint, mint_vtoken, transfer_token,
};
use crate::state::{
    Deposited, StarkeConfig, StarkeConfigError, TokenWhitelist, TokenWhitelistError, UserWhitelist,
    UserWhitelistError, Vault, VaultDepositFeeConfig, VaultDepositFeeConfigError, VaultError, VaultState,
};

pub fn _deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, Deposit<'info>>,
    amount: u64,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    require!(
        ctx.accounts.vault.state != VaultState::DepositPaused,
        VaultError::DepositsPaused
    );

    msg!("Processing deposit request of {} tokens", amount);
    msg!("User: {}", ctx.accounts.user.key());
    msg!("Vault: {}", ctx.accounts.vault.key());
    msg!(
        "Deposit token mint: {}",
        ctx.accounts.deposit_token_mint.key()
    );

    // Validate user is whitelisted and get their investor type
    let investor_type = ctx
        .accounts
        .user_whitelist
        .get_user_type(&ctx.accounts.user.key())
        .ok_or(UserWhitelistError::UserNotWhitelisted)?;
    msg!("User investor type: {:?}", investor_type);

    // Validate investor type is allowed for this vault
    ctx.accounts.vault.validate_investor_type(&investor_type)?;

    // Validate deposit amount based on investor type
    ctx.accounts
        .vault
        .validate_deposit_amount_by_type(amount, &investor_type)?;

    // Check if this is a new depositor (first time depositing)
    let is_new_depositor = ctx.accounts.vtoken_account.amount == 0;

    // Validate max depositors limit
    ctx.accounts
        .vault
        .validate_max_depositors(is_new_depositor)?;

    // Calculate deposit fee if enabled
    // First, verify the deposit_fee_config PDA if it exists
    let (fee_amount, net_deposit_amount, fee_recipient) = {
        let expected_pda = Pubkey::find_program_address(
            &[VaultDepositFeeConfig::SEED, ctx.accounts.vault.key().as_ref()],
            ctx.program_id,
        );
        
        if ctx.accounts.deposit_fee_config.key() != expected_pda.0 {
            // Account doesn't exist or is wrong address
            msg!("Deposit fee config not found at expected PDA");
            (0, amount, None)
        } else if ctx.accounts.deposit_fee_config.data_is_empty() {
            msg!("Deposit fee config not initialized");
            (0, amount, None)
        } else {
            match VaultDepositFeeConfig::try_deserialize(
                &mut &ctx.accounts.deposit_fee_config.data.borrow()[..],
            ) {
                Ok(fee_config) => {
                    if fee_config.is_initialized() && fee_config.enabled {
                        // Validate that the platform fee recipient token account matches the config
                        require_keys_eq!(
                            ctx.accounts.platform_fee_recipient_token_account.owner,
                            fee_config.platform_fee_recipient,
                            VaultDepositFeeConfigError::PlatformFeeRecipientMismatch
                        );
                        // Validate that the token account mint matches the deposit token mint
                        require_keys_eq!(
                            ctx.accounts.platform_fee_recipient_token_account.mint,
                            ctx.accounts.deposit_token_mint.key(),
                            VaultDepositFeeConfigError::PlatformFeeRecipientMismatch
                        );
                        
                        let fee = fee_config.calculate_fee_amount(amount)?;
                        // Defensive check: fee should never exceed deposit amount
                        // This would only happen if fee_rate > 10000, which is prevented at initialization
                        require!(
                            fee <= amount,
                            VaultDepositFeeConfigError::InvalidFeeRate
                        );
                        let net = amount - fee;
                        msg!("Deposit fee enabled: fee_amount={}, net_deposit_amount={}, recipient={}", 
                             fee, net, fee_config.platform_fee_recipient);
                        (fee, net, Some(fee_config.platform_fee_recipient))
                    } else {
                        msg!("Deposit fee not enabled or not configured");
                        (0, amount, None)
                    }
                }
                Err(_) => {
                    // Account exists but can't be deserialized (shouldn't happen)
                    msg!("Deposit fee config exists but deserialization failed");
                    (0, amount, None)
                }
            }
        }
    };

    // Calculate the total AUM using vault's get_aum function
    let total_aum = ctx.accounts.vault.get_aum(
        ctx.remaining_accounts,
        &ctx.accounts.token_whitelist,
        &ctx.accounts.vault.key(),
    )?;
    msg!("Vault AUM: {}", total_aum);

    // Calculate the USD value of deposit tokens (using net deposit amount after fee)
    let deposit_value = calculate_deposit_token_value(
        &ctx.accounts.token_whitelist,
        ctx.accounts.deposit_token_mint.key(),
        ctx.accounts.deposit_token_mint.decimals,
        net_deposit_amount,
        &ctx.accounts.deposit_token_price_update,
    )?;
    msg!("Deposit value (net after fee): {}", deposit_value);

    // Validate max AUM for private vaults
    ctx.accounts
        .vault
        .validate_max_aum(total_aum, deposit_value)?;

    // Calculate vtokens to mint based on AUM
    let vtokens_to_mint = calculate_vtokens_to_mint(
        total_aum,
        deposit_value,
        ctx.accounts.vtoken_mint.supply,
        ctx.accounts.vault.initial_vtoken_price,
    )?;
    msg!("Vtokens to mint: {}", vtokens_to_mint);

    // Transfer deposit fee to platform if enabled
    if fee_amount > 0 && fee_recipient.is_some() {
        transfer_token(
            &ctx.accounts.user_deposit_token_account,
            &ctx.accounts.platform_fee_recipient_token_account,
            fee_amount,
            &ctx.accounts.deposit_token_mint,
            &ctx.accounts.user,
            &ctx.accounts.token_program,
        )?;
        msg!(
            "{} tokens transferred as fee to platform recipient: {}",
            fee_amount,
            ctx.accounts.platform_fee_recipient_token_account.key()
        );
    }

    // Transfer net deposit tokens from depositor to vault (amount after fee)
    transfer_token(
        &ctx.accounts.user_deposit_token_account,
        &ctx.accounts.vault_deposit_token_account,
        net_deposit_amount,
        &ctx.accounts.deposit_token_mint,
        &ctx.accounts.user,
        &ctx.accounts.token_program,
    )?;
    msg!(
        "{} tokens (net after fee) transferred from user to vault successfully",
        net_deposit_amount
    );

    // Mint vtokens to depositor
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
    msg!("{} vtokens minted to user successfully", vtokens_to_mint);

    // Increment depositor count if this is a new depositor
    if is_new_depositor {
        ctx.accounts.vault.increment_depositor_count()?;
        msg!(
            "New depositor added. Total depositors: {}",
            ctx.accounts.vault.current_depositors
        );
    }

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
        address = STARKE_AUTHORITY @ TokenWhitelistError::UnauthorizedAccess,
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
        associated_token::token_program = token_2022_program,
    )]
    pub vtoken_account: Box<InterfaceAccount<'info, TokenAccount>>,

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

    // Deposit token mint
    #[account(
        constraint = deposit_token_mint.key() == vault.deposit_token_mint,
    )]
    pub deposit_token_mint: Box<InterfaceAccount<'info, Mint>>,

    // Deposit token price update
    pub deposit_token_price_update: Box<Account<'info, PriceUpdateV2>>,

    // Token whitelist
    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = token_whitelist.bump,
    )]
    pub token_whitelist: Box<Account<'info, TokenWhitelist>>,

    // User whitelist
    #[account(
        seeds = [UserWhitelist::SEED],
        bump = user_whitelist.bump,
    )]
    pub user_whitelist: Box<Account<'info, UserWhitelist>>,

    // Starke config
    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    // Deposit fee config (optional - can be uninitialized or not exist)
    /// CHECK: Account may not exist or be uninitialized, we check in the function
    pub deposit_fee_config: UncheckedAccount<'info>,

    // Platform fee recipient token account (required but only used if fee is enabled)
    /// CHECK: Account is validated in the function to match the fee config's recipient
    #[account(
        mut,
    )]
    pub platform_fee_recipient_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub clock: Sysvar<'info, Clock>,
    // Used for deposit token mint
    pub token_program: Interface<'info, TokenInterface>,
    // Used for vtoken mint
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
