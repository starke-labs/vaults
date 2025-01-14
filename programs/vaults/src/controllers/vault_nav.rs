use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{
    constants::NAV_DECIMALS,
    state::{TokenWhitelist, VaultError},
};

pub struct VaultTokenInfo<'info> {
    pub token_balance: u64,
    pub token_decimals: u8,
    pub price_feed_id: String,
    pub price_update: Box<Account<'info, PriceUpdateV2>>,
}

pub fn parse_vault_balances<'info>(
    remaining_accounts: &'info [AccountInfo<'info>],
    whitelist: Box<Account<'info, TokenWhitelist>>,
    vault_key: Pubkey,
) -> Result<Vec<VaultTokenInfo<'info>>> {
    let mut vault_token_infos = Vec::new();

    for chunk in remaining_accounts.chunks(3) {
        // Each chunk must contain 3 accounts in this order:
        // 1. Token mint
        // 2. Token account
        // 3. Price update
        let mint = Account::<'info, Mint>::try_from(&chunk[0])?;
        let token_account: Account<'info, TokenAccount> = Account::try_from(&chunk[1])?;
        let price_update: Box<Account<'info, PriceUpdateV2>> =
            Box::new(Account::try_from(&chunk[2])?);

        require!(
            mint.key() == token_account.mint,
            VaultError::MintAndTokenAccountMismatch
        );
        require!(
            vault_key == token_account.owner,
            VaultError::VaultAndTokenAccountMismatch
        );

        let price_feed_id = whitelist.get_price_feed_id(&mint.key())?;

        vault_token_infos.push(VaultTokenInfo {
            token_balance: token_account.amount,
            token_decimals: mint.decimals,
            price_feed_id,
            price_update,
        });
    }

    Ok(vault_token_infos)
}

// TODO: Consider using u128 for the value
pub fn compute_token_value_usd(
    token_balance: u64,
    token_decimals: u8,
    price_in_nav_decimals: u64,
) -> Result<u64> {
    msg!("compute_token_value_usd called");
    msg!("Token balance: {}", token_balance);
    msg!("Token decimals: {}", token_decimals);
    msg!("Price in NAV decimals: {}", price_in_nav_decimals);

    let value = token_balance
        .checked_mul(price_in_nav_decimals)
        .ok_or(VaultError::NumericOverflow)?
        .checked_div(10u64.pow(token_decimals as u32))
        .ok_or(VaultError::NumericOverflow)?;
    msg!("Token USD value: {}", value);

    Ok(value)
}
