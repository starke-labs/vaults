use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

pub fn _test_pyth_price_feed(ctx: Context<TestPythPriceFeed>) -> Result<()> {
    let price_update = &mut ctx.accounts.price_update;

    // get_price_no_older_than will fail if the price update is more than 30 seconds old
    let maximum_age: u64 = 30;

    // get_price_no_older_than will fail if the price update is for a different price feed.
    // This string is the id of the BTC/USD feed. See https://pyth.network/developers/price-feed-ids for all available IDs.
    let feed_id: [u8; 32] =
        get_feed_id_from_hex("0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43")?;
    let price = price_update.get_price_no_older_than(&Clock::get()?, maximum_age, &feed_id)?;

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

    pub price_update: Account<'info, PriceUpdateV2>,
}
