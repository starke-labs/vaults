use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::{
    constants::NAV_DECIMALS,
    state::{TokenWhitelist, VaultError},
};

pub struct VaultTokenInfo {
    pub token_balance: u64,
    pub token_decimals: u8,
    pub price_feed_id: String,
}

pub fn parse_vault_balances<'info>(
    remaining_accounts: &'info [AccountInfo<'info>],
    whitelist: Account<'info, TokenWhitelist>,
    vault_key: Pubkey,
) -> Result<Vec<VaultTokenInfo>> {
    let mut vault_token_infos = Vec::new();

    for chunk in remaining_accounts.chunks(2) {
        let mint = Account::<'info, Mint>::try_from(&chunk[0])?;
        let token_account: Account<'info, TokenAccount> = Account::try_from(&chunk[1])?;

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
        });
    }

    Ok(vault_token_infos)
}

// TODO: consider using u128 for the value
pub fn compute_token_value_usd(token_balance: u64, token_decimals: u8, price: u64) -> Result<u64> {
    let token_balance_in_nav_decimals = token_balance
        .checked_mul(10u64.pow((NAV_DECIMALS - token_decimals) as u32))
        .ok_or(VaultError::NumericOverflow)?;
    let value = token_balance_in_nav_decimals
        .checked_mul(price)
        .ok_or(VaultError::NumericOverflow)?;

    Ok(value)
}
