use anchor_lang::prelude::*;
use anchor_spl::token::*;

use crate::constants::*;
use crate::state::*;

pub fn _create_vault(ctx: Context<CreateVault>, name: String) -> Result<()> {
    ctx.accounts.vault.initialize(
        *ctx.accounts.manager.key,
        ctx.accounts.deposit_token.key(),
        ctx.accounts.vault_token_mint.key(),
        name,
        ctx.bumps.vault,
    )?;

    // TODO: Remove this as we are initializing the mint in accounts
    // let cpi_accounts = InitializeMint {
    //     mint: ctx.accounts.vault_token_mint.to_account_info(),
    //     rent: ctx.accounts.rent.to_account_info(),
    // };
    // let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    // initialize_mint(
    //     cpi_ctx,
    //     6,
    //     &ctx.accounts.vault.key(),
    //     Some(&ctx.accounts.vault.key()),
    // )?;

    emit!(VaultCreated {
        vault: ctx.accounts.vault.key(),
        manager: *ctx.accounts.manager.key,
        deposit_token: ctx.accounts.deposit_token.key(),
        name: ctx.accounts.vault.name.to_string(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    // Vault manager
    #[account(mut)]
    pub manager: Signer<'info>,

    // Vault (PDA)
    // Currently seeded by manager pubkey, which means that the manager can only create one vault
    // This also means when users want to deposit, they need to send manager's pubkey in accounts
    // Do we want this? Or should we avoid PDA here and store vault pubkeys instead?
    // TODO: Find a way to allow multiple vaults per manager for v1
    #[account(
        init,
        payer = manager,
        space = Vault::MAX_SPACE,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,

    // Vault's SPL token mint account (PDA)
    #[account(
        init,
        payer = manager,
        seeds = [Vault::VAULT_TOKEN_MINT_SEED, vault.key().as_ref()],
        bump,
        // NOTE: We can not change the deposit token after initializing the vault (which also initializes the mint)
        mint::decimals = deposit_token.decimals,
        mint::authority = vault,
        mint::freeze_authority = vault,
    )]
    pub vault_token_mint: Account<'info, Mint>,

    #[account(
        seeds = [TOKEN_WHITELIST_SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Account<'info, TokenWhitelist>,

    #[account(
        constraint = whitelist.is_whitelisted(&deposit_token.key()) @ WhitelistError::TokenNotWhitelisted,
    )]
    pub deposit_token: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}
