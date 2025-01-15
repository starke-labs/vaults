use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::controllers::{
    compute_token_value_usd, get_token_price_from_pyth_feed, transform_price_to_nav_decimals,
};
use crate::state::TokenWhitelist;

pub fn calculate_deposit_token_value<'info>(
    whitelist: &Account<'info, TokenWhitelist>,
    deposit_token_mint: &Pubkey,
    deposit_token_decimals: u8,
    amount: u64,
    price_update: Box<Account<'info, PriceUpdateV2>>,
) -> Result<u64> {
    // msg!("calculate_deposit_token_value called");
    // msg!("Deposit token mint: {}", deposit_token_mint);
    // msg!("Deposit token decimals: {}", deposit_token_decimals);
    // msg!("Deposit amount: {}", amount);
    let deposit_price_feed_id = whitelist.get_price_feed_id(deposit_token_mint)?;
    let deposit_price = get_token_price_from_pyth_feed(deposit_price_feed_id, price_update)?;
    // msg!(
    //     "Deposit price: {} {} {}",
    //     deposit_price.price,
    //     deposit_price.conf,
    //     deposit_price.exponent
    // );

    let deposit_price_in_nav_decimals = transform_price_to_nav_decimals(deposit_price)?;
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
