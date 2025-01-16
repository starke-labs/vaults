use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::controllers::get_token_price_from_pyth_feed;

pub fn _test_pyth_price_feed(ctx: Context<TestPythPriceFeed>) -> Result<()> {
    let price_feed_id = "0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43";
    let price = get_token_price_from_pyth_feed(price_feed_id, &ctx.accounts.price_update)?;

    msg!(
        "The price is ({} ± {}) * 10^{}",
        price.price,
        price.conf,
        price.exponent
    );

    Ok(())
}

#[derive(Accounts)]
pub struct TestPythPriceFeed<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub price_update: Box<Account<'info, PriceUpdateV2>>,
}
