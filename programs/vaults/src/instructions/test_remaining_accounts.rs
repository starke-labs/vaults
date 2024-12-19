use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::state::{TokenWhitelist, Vault};

pub fn _test_remaining_accounts<'info>(
    ctx: Context<'_, '_, 'info, 'info, TestRemainingAccounts<'info>>,
) -> Result<()> {
    msg!(
        "Remaining accounts length: {}",
        ctx.remaining_accounts.len()
    );

    for chunk in ctx.remaining_accounts.chunks(2) {
        let mint = Account::<'info, Mint>::try_from(&chunk[0])?;
        let token_account: Account<'info, TokenAccount> = Account::try_from(&chunk[1])?;

        require!(
            mint.key() == token_account.mint,
            TestRemainingAccountsError::MintAndTokenAccountMismatch
        );
        require!(
            ctx.accounts.vault.key() == token_account.owner,
            TestRemainingAccountsError::VaultAndTokenAccountMismatch
        );

        msg!("Mint: {:?}", mint.key());
        msg!("Mint decimals: {:?}", mint.decimals);
        msg!("Token account: {:?}", token_account.key());
        msg!("Token account balance: {:?}", token_account.amount);
        msg!("Token account owner: {:?}", token_account.owner);

        let price_feed_id = ctx.accounts.whitelist.get_price_feed_id(&mint.key())?;
        msg!("Price feed id: {:?}", price_feed_id);
    }

    Ok(())
}

#[derive(Accounts)]
pub struct TestRemainingAccounts<'info> {
    pub signer: Signer<'info>,

    /// CHECK: We can skip checking the manager
    pub manager: UncheckedAccount<'info>,

    #[account(
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        seeds = [TokenWhitelist::SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Account<'info, TokenWhitelist>,
}

#[error_code]
pub enum TestRemainingAccountsError {
    #[msg("Mint and token account mismatch")]
    MintAndTokenAccountMismatch,

    #[msg("Vault and token account mismatch")]
    VaultAndTokenAccountMismatch,
}
