use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use jupiter::program::Jupiter;

use crate::{
    constants::STARKE_AUTHORITY,
    state::{
        StarkeConfig, StarkeConfigError, TokenWhitelist, TokenWhitelistError, Vault, VaultError,
    },
};

pub fn _swap_to_deposit_token_on_jupiter(
    ctx: Context<SwapToDepositTokenOnJupiter>,
    data: Vec<u8>,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    msg!("Processing Jupiter swap request");
    msg!("Manager: {}", ctx.accounts.manager.key());
    msg!("Vault: {}", ctx.accounts.vault.key());
    msg!("Input token: {}", ctx.accounts.input_token_mint.key());
    msg!("Deposit token: {}", ctx.accounts.vault.deposit_token_mint);
    msg!("Jupiter program: {}", ctx.accounts.jupiter_program.key);
    msg!("Data size: {} bytes", data.len());

    let accounts: Vec<AccountMeta> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| AccountMeta {
            pubkey: *acc.key,
            is_signer: acc.key == &ctx.accounts.vault.key() || acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();
    msg!("Account metas prepared: {} accounts", accounts.len());

    let accounts_infos: Vec<AccountInfo> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| AccountInfo {
            key: acc.key,
            is_signer: acc.key == &ctx.accounts.vault.key() || acc.is_signer,
            is_writable: acc.is_writable,
            lamports: acc.lamports.clone(),
            data: acc.data.clone(),
            owner: acc.owner,
            rent_epoch: acc.rent_epoch,
            executable: acc.executable,
        })
        .collect();
    msg!("Account infos prepared: {} accounts", accounts_infos.len());

    let manager = ctx.accounts.manager.key();
    let signer_seeds: &[&[&[u8]]] = &[&[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]]];

    invoke_signed(
        &Instruction {
            program_id: *ctx.accounts.jupiter_program.key,
            accounts,
            data,
        },
        &accounts_infos,
        signer_seeds,
    )?;

    msg!("Jupiter swap to deposit token completed successfully");

    Ok(())
}

#[derive(Accounts)]
pub struct SwapToDepositTokenOnJupiter<'info> {
    #[account(mut, address = STARKE_AUTHORITY @ VaultError::Unauthorized)]
    pub starke_authority: Signer<'info>,

    /// CHECK: Manager is not validated directly; it is used as a seed for the vault PDA,
    /// so passing an incorrect manager will cause the vault seeds constraint to fail.
    pub manager: UncheckedAccount<'info>,

    // Vault
    #[account(
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    // Token whitelist
    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = token_whitelist.bump,
    )]
    pub token_whitelist: Box<Account<'info, TokenWhitelist>>,

    // Token input mint
    #[account(
        constraint = token_whitelist.is_whitelisted(&input_token_mint.key()) @ TokenWhitelistError::TokenNotWhitelisted
    )]
    pub input_token_mint: Box<InterfaceAccount<'info, Mint>>,

    // Token input account
    #[account(
        mut,
        associated_token::authority = vault,
        associated_token::mint = input_token_mint
    )]
    pub input_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // Deposit token mint (output of the swap)
    #[account(
        constraint = deposit_token_mint.key() == vault.deposit_token_mint @ VaultError::InvalidDepositToken
    )]
    pub deposit_token_mint: Box<InterfaceAccount<'info, Mint>>,

    // Vault's deposit token account (destination of the swap)
    #[account(
        mut,
        associated_token::authority = vault,
        associated_token::mint = deposit_token_mint
    )]
    pub vault_deposit_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    // Starke config
    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    pub jupiter_program: Program<'info, Jupiter>,
    pub token_program: Interface<'info, TokenInterface>,
}
