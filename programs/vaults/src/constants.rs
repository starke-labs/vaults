use anchor_lang::prelude::*;

pub static STARKE_AUTHORITY: Pubkey = pubkey!("STRK1me6eFLDYGKYqbn2oyHsaxiCHe8GDWQnnSGiScS");

// Can be as low as 30 seconds because we are using the Pyth sponsored price feed
pub static PYTH_PRICE_FEED_MAX_AGE_SECONDS: u64 = 30;

// 100 basis points = 1%
pub static PYTH_CONFIDENCE_THRESHOLD_BPS: u64 = 100;

pub static AUM_DECIMALS: u8 = 9;

pub static PRECISION: u64 = 10u64.pow(AUM_DECIMALS as u32);
