use anchor_lang::prelude::*;

pub static PROGRAM_AUTHORITY: Pubkey = pubkey!("STRK1me6eFLDYGKYqbn2oyHsaxiCHe8GDWQnnSGiScS");

// TODO: 30 fails, need to check if 60 is good enough to be safe
pub static PYTH_PRICE_FEED_MAX_AGE_SECONDS: u64 = 60;

pub static NAV_DECIMALS: u8 = 9;
