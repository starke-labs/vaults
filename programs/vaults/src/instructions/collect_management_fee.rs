use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};
use crate::{
    constants::STARKE_AUTHORITY,
    controllers::{calculate_management_fee_vtokens, mint_vtoken},
    state::{ManagementFeeCollected, StarkeConfig, StarkeConfigError, Vault, VaultError},
};

pub fn _collect_management_fee(ctx: Context<CollectManagementFee>) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );
    require_keys_eq!(
        ctx.accounts.authority.key(),
        STARKE_AUTHORITY,
        StarkeConfigError::Unauthorized
    );

    let total_supply = ctx.accounts.vtoken_mint.supply;
    require!(total_supply > 0, VaultError::NoVtokenSupply);

    // Calculate 0.25% (25 basis points) of total supply
    let fee_amount = calculate_management_fee_vtokens(total_supply)?;
    require!(fee_amount > 0, VaultError::NoVtokenSupply);

    let manager = ctx.accounts.manager.key();
    let signer_seeds: &[&[&[u8]]] = &[&[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]]];

    mint_vtoken(
        &ctx.accounts.vault,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.fee_recipient_vtoken_account,
        fee_amount,
        signer_seeds,
        &ctx.accounts.token_2022_program,
    )?;

    let new_supply = total_supply
        .checked_add(fee_amount)
        .ok_or(error!(VaultError::NumericOverflow))?;

    emit!(ManagementFeeCollected {
        vault: ctx.accounts.vault.key(),
        authority: ctx.accounts.authority.key(),
        recipient: ctx.accounts.fee_recipient.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        vtoken_fee_amount: fee_amount,
        new_vtoken_supply: new_supply,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CollectManagementFee<'info> {
    #[account(
        mut,
        address = STARKE_AUTHORITY @ StarkeConfigError::Unauthorized,
    )]
    pub authority: Signer<'info>,

    /// CHECK: Manager is only used for PDA derivation
    pub manager: UncheckedAccount<'info>,

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

    /// CHECK: Fee recipient may be any address controlled by the general partner
    pub fee_recipient: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::authority = fee_recipient,
        associated_token::mint = vtoken_mint,
        associated_token::token_program = token_2022_program,
    )]
    pub fee_recipient_vtoken_account: Box<InterfaceAccount<'info, TokenAccount>>,

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

