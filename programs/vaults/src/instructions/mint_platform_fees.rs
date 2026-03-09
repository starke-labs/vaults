use crate::{
    controllers::{calculate_platform_fees_vtokens_to_mint, mint_vtoken},
    state::{PlatformFeeMinted, StarkeConfig, StarkeConfigError, Vault, VaultError},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

pub fn _mint_platform_fees(ctx: Context<MintPlatformFees>) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    msg!("Processing platform fees minting request");
    msg!("Vault: {}", ctx.accounts.vault.key());
    msg!(
        "Platform fees rate: {} bps",
        ctx.accounts.vault.platform_fees_rate
    );
    msg!(
        "Recipient: {}",
        ctx.accounts.platform_fees_recipient.key()
    );

    // Check if platform fees are enabled and due
    require!(
        ctx.accounts.vault.platform_fees_rate > 0
            && ctx
                .accounts
                .vault
                .can_pay_platform_fees(ctx.accounts.clock.unix_timestamp),
        VaultError::FeesNotDue
    );

    let current_vtoken_supply = ctx.accounts.vtoken_mint.supply;
    require!(current_vtoken_supply > 0, VaultError::NoVtokenSupply);

    let vtokens_to_mint = calculate_platform_fees_vtokens_to_mint(
        current_vtoken_supply,
        ctx.accounts.vault.platform_fees_rate,
    )?;
    require!(vtokens_to_mint > 0, VaultError::NoVtokenSupply);

    msg!("vTokens to mint as platform fees: {}", vtokens_to_mint);

    let manager = ctx.accounts.manager.key();
    let signer_seeds: &[&[&[u8]]] = &[&[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]]];

    mint_vtoken(
        &ctx.accounts.vault,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.platform_fees_recipient_vtoken_account,
        vtokens_to_mint,
        signer_seeds,
        &ctx.accounts.token_2022_program,
    )?;

    msg!(
        "{} vTokens minted to platform fees recipient successfully",
        vtokens_to_mint
    );

    // Update last payment timestamp
    ctx.accounts.vault.last_platform_fees_paid_timestamp = ctx.accounts.clock.unix_timestamp;

    let new_supply = current_vtoken_supply
        .checked_add(vtokens_to_mint)
        .ok_or(error!(VaultError::NumericOverflow))?;

    emit!(PlatformFeeMinted {
        vault: ctx.accounts.vault.key(),
        recipient: ctx.accounts.platform_fees_recipient.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        vtoken_fee_amount: vtokens_to_mint,
        new_vtoken_supply: new_supply,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    msg!("Platform fees minted successfully");

    Ok(())
}

#[derive(Accounts)]
pub struct MintPlatformFees<'info> {
    // Platform fees recipient (must sign and pay for ATA creation if needed)
    #[account(mut)]
    pub platform_fees_recipient: Signer<'info>,

    // Manager (for vault PDA derivation)
    /// CHECK: Used for PDA derivation only
    pub manager: UncheckedAccount<'info>,

    // Vault
    #[account(
        mut,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    // vToken mint
    #[account(
        mut,
        seeds = [Vault::VTOKEN_MINT_SEED, vault.key().as_ref()],
        bump = vault.mint_bump,
        mint::token_program = token_2022_program,
        constraint = vault.mint == vtoken_mint.key() @ VaultError::InvalidVtokenMint
    )]
    pub vtoken_mint: Box<InterfaceAccount<'info, Mint>>,

    // Platform fees recipient's vToken account (created if needed)
    #[account(
        init_if_needed,
        payer = platform_fees_recipient,
        associated_token::authority = platform_fees_recipient,
        associated_token::mint = vtoken_mint,
        associated_token::token_program = token_2022_program,
    )]
    pub platform_fees_recipient_vtoken_account: Box<InterfaceAccount<'info, TokenAccount>>,

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



