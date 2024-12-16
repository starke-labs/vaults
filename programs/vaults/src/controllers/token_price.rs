use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, Price, PriceUpdateV2};

use crate::constants::PYTH_PRICE_FEED_MAX_AGE_SECONDS;

pub fn get_token_price_from_pyth_feed<'info>(
    price_feed_id: String,
    // TODO: check if the price_update type is correct (pyth example had `&mut Account<'info, PriceUpdateV2>`)
    price_update: Account<'info, PriceUpdateV2>,
) -> Result<Price> {
    let feed_id: [u8; 32] = get_feed_id_from_hex(&price_feed_id)?;
    let price = price_update.get_price_no_older_than(
        &Clock::get()?,
        PYTH_PRICE_FEED_MAX_AGE_SECONDS,
        &feed_id,
    )?;

    Ok(price)
}
