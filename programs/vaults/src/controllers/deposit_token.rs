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
    // msg!("calculate_deposit_token_value called");
    // msg!("Deposit token mint: {}", deposit_token_mint);
    // msg!("Deposit token decimals: {}", deposit_token_decimals);
    // msg!("Deposit amount: {}", amount);
    // let deposit_price_feed_id = whitelist.get_price_feed_id(&deposit_token_mint)?;
    let deposit_price =
        verify_price_update_and_get_pyth_price(whitelist, &deposit_token_mint, price_update)?;
    // msg!(
    //     "Deposit price: {} {} {}",
    //     deposit_price.price,
    //     deposit_price.conf,
    //     deposit_price.exponent
    // );

    let deposit_price_in_nav_decimals = transform_price_to_nav_decimals(&deposit_price)?;
    // msg!(
    //     "Deposit price in nav decimals: {}",
    //     deposit_price_in_nav_decimals
    // );
    compute_token_value_usd(
        amount,
        deposit_token_decimals,
        deposit_price_in_nav_decimals,
    )
}
