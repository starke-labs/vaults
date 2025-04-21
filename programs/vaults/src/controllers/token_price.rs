use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, Price, PriceUpdateV2};

use crate::constants::{PRECISION, PYTH_PRICE_FEED_MAX_AGE_SECONDS};
use crate::state::VaultError;

pub fn get_token_price_from_pyth_feed<'info>(
    price_feed_id: &str,
    price_update: &Account<'info, PriceUpdateV2>,
) -> Result<Price> {
    let feed_id: [u8; 32] = get_feed_id_from_hex(&price_feed_id)?;
    let price = price_update.get_price_no_older_than(
        &Clock::get()?,
        PYTH_PRICE_FEED_MAX_AGE_SECONDS,
        &feed_id,
    )?;

    // The price is in the range (p-c, p+c), where p is `price.price` and c is `price.conf`.
    // We want to check if c / p is less than 0.01 (1%)
    // Scale up by 10000 to handle 4 decimal places in the percentage calculation
    let conf_ratio = price
        .conf
        .checked_mul(10u64.pow(4))
        .ok_or(VaultError::NumericOverflow)?
        .checked_div(price.price.abs() as u64)
        .ok_or(VaultError::NumericOverflow)?;
    require!(conf_ratio < 100, VaultError::PriceConfidenceTooLow); // 100 basis points = 1%

    Ok(price)
}

pub fn transform_price_to_nav_decimals(price: &Price) -> Result<u64> {
    // TODO: Handle case when exponent is positive or check if that is possible in pyth price feeds?
    Ok(price
        .price
        .unsigned_abs()
        .checked_mul(PRECISION)
        .ok_or(VaultError::NumericOverflow)?
        .checked_div(10u64.pow((-price.exponent) as u32))
        .ok_or(VaultError::NumericOverflow)?)
}
