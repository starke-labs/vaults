use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, Price, PriceUpdateV2};

use crate::constants::{NAV_DECIMALS, PYTH_PRICE_FEED_MAX_AGE_SECONDS};
use crate::state::VaultError;

pub fn get_token_price_from_pyth_feed(
    price_feed_id: String,
    price_update: Account<PriceUpdateV2>,
) -> Result<Price> {
    let feed_id: [u8; 32] = get_feed_id_from_hex(&price_feed_id)?;
    let price = price_update.get_price_no_older_than(
        &Clock::get()?,
        PYTH_PRICE_FEED_MAX_AGE_SECONDS,
        &feed_id,
    )?;

    Ok(price)
}

// TODO: check the calculation
pub fn transform_price_to_nav_decimals(price: Price) -> Result<u64> {
    Ok(price
        .price
        .unsigned_abs()
        .checked_mul(10u64.pow((NAV_DECIMALS - price.exponent as u8) as u32))
        .ok_or(VaultError::NumericOverflow)?)
}
