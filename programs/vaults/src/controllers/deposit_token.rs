use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use super::{
    compute_token_value_usd, transform_price_to_nav_decimals,
    verify_price_update_and_get_pyth_price,
};
use crate::state::TokenWhitelist;

pub fn calculate_deposit_token_value<'info>(
    whitelist: &Account<'info, TokenWhitelist>,
    deposit_token_mint: Pubkey,
    deposit_token_decimals: u8,
    amount: u64,
    price_update: &Account<'info, PriceUpdateV2>,
) -> Result<u64> {
    let deposit_price =
        verify_price_update_and_get_pyth_price(whitelist, &deposit_token_mint, price_update)?;
    let deposit_price_in_nav_decimals = transform_price_to_nav_decimals(&deposit_price)?;

    compute_token_value_usd(
        amount,
        deposit_token_decimals,
        deposit_price_in_nav_decimals,
    )
}
