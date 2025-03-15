use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::jupiter::Jupiter;
use crate::state::{TokenWhitelist, Vault, WhitelistError};

pub fn _swap_on_jupiter(ctx: Context<SwapOnJupiter>, data: Vec<u8>) -> Result<()> {
    let accounts: Vec<AccountMeta> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| AccountMeta {
            pubkey: *acc.key,
            is_signer: acc.key == &ctx.accounts.vault.key() || acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let accounts_infos: Vec<AccountInfo> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| {
            let is_signer = acc.key == &ctx.accounts.vault.key() || acc.is_signer;
            AccountInfo {
                key: acc.key,
                is_signer,
                is_writable: acc.is_writable,
                lamports: acc.lamports.clone(),
                data: acc.data.clone(),
                owner: acc.owner,
                rent_epoch: acc.rent_epoch,
                executable: acc.executable,
            }
        })
        .collect();

    let manager = ctx.accounts.manager.key();
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    let signer_seeds = &[&vault_seeds[..]];

    invoke_signed(
        &Instruction {
            program_id: *ctx.accounts.jupiter_program.key,
            accounts,
            data,
        },
        &accounts_infos,
        signer_seeds,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct SwapOnJupiter<'info> {
    // Manager
    #[account(mut)]
    pub manager: Signer<'info>,

    // Vault
    #[account(
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    // Whitelist
    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Box<Account<'info, TokenWhitelist>>,

    // Token input mint
    #[account(
        constraint = whitelist.is_whitelisted(&input_token_mint.key()) @ WhitelistError::TokenNotWhitelisted
    )]
    pub input_token_mint: Box<Account<'info, Mint>>,

    // Token output mint
    #[account(
        constraint = whitelist.is_whitelisted(&output_token_mint.key()) @ WhitelistError::TokenNotWhitelisted
    )]
    pub output_token_mint: Box<Account<'info, Mint>>,

    // Token input account
    #[account(
        mut,
        associated_token::authority = vault,
        associated_token::mint = input_token_mint
    )]
    pub input_token_account: Box<Account<'info, TokenAccount>>,

    // Token output account
    #[account(
        init_if_needed,
        payer = manager,
        associated_token::authority = vault,
        associated_token::mint = output_token_mint
    )]
    pub output_token_account: Box<Account<'info, TokenAccount>>,

    pub jupiter_program: Program<'info, Jupiter>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
