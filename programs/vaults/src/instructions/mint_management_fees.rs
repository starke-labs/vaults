use crate::{
    controllers::{calculate_management_fees_vtokens_to_mint, mint_vtoken},
    state::{ManagementFeeMinted, StarkeConfig, StarkeConfigError, Vault, VaultError},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

pub fn _mint_management_fees(ctx: Context<MintManagementFees>) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );
    require!(
        ctx.accounts.vault.management_fees_rate > 0
            && ctx
                .accounts
                .vault
                .can_pay_management_fees(ctx.accounts.clock.unix_timestamp),
        VaultError::FeesNotDue
    );

    let current_vtoken_supply = ctx.accounts.vtoken_mint.supply;
    require!(current_vtoken_supply > 0, VaultError::NoVtokenSupply);

    let vtokens_to_mint = calculate_management_fees_vtokens_to_mint(
        current_vtoken_supply,
        ctx.accounts.vault.management_fees_rate,
    )?;
    require!(vtokens_to_mint > 0, VaultError::NoVtokenSupply);

    let manager = ctx.accounts.manager.key();
    let signer_seeds: &[&[&[u8]]] = &[&[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]]];

    mint_vtoken(
        &ctx.accounts.vault,
        &ctx.accounts.vtoken_mint,
        &ctx.accounts.manager_vtoken_account,
        vtokens_to_mint,
        signer_seeds,
        &ctx.accounts.token_2022_program,
    )?;

    let new_supply = current_vtoken_supply
        .checked_add(vtokens_to_mint)
        .ok_or(error!(VaultError::NumericOverflow))?;

    emit!(ManagementFeeMinted {
        vault: ctx.accounts.vault.key(),
        manager: ctx.accounts.manager.key(),
        vtoken_mint: ctx.accounts.vtoken_mint.key(),
        vtoken_fee_amount: vtokens_to_mint,
        new_vtoken_supply: new_supply,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct MintManagementFees<'info> {
    // Manager
    #[account(mut)]
    pub manager: Signer<'info>,

    // Vault
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
        constraint = vault.mint == vtoken_mint.key() @ VaultError::InvalidVtokenMint
    )]
    pub vtoken_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = manager,
        associated_token::authority = manager,
        associated_token::mint = vtoken_mint,
        associated_token::token_program = token_2022_program,
    )]
    pub manager_vtoken_account: Box<InterfaceAccount<'info, TokenAccount>>,

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
