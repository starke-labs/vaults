use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{Price, PriceUpdateV2};

use crate::controllers::get_token_price_from_pyth_feed;
use crate::state::TokenWhitelist;

pub fn verify_price_update_and_get_pyth_price<'info>(
    whitelist: &Account<'info, TokenWhitelist>,
    token_mint: &Pubkey,
    price_update: &Account<'info, PriceUpdateV2>,
) -> Result<Price> {
    whitelist.verify_price_update(token_mint, &price_update.key())?;
    let price_feed_id = whitelist.get_price_feed_id(token_mint)?;
    get_token_price_from_pyth_feed(price_feed_id, price_update)
}
