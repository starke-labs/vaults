use anchor_lang::prelude::*;

#[cfg(not(feature = "devnet"))]
pub static STARKE_AUTHORITY: Pubkey = pubkey!("STRK1me6eFLDYGKYqbn2oyHsaxiCHe8GDWQnnSGiScS");

#[cfg(feature = "devnet")]
pub static STARKE_AUTHORITY: Pubkey = pubkey!("H2RYjGrqbq1EgcW3vrNF7UkHN76zem7VATKnvPNMuTH4");

// Can be as low as 60 seconds because we are using the Pyth sponsored price feed
pub static PYTH_PRICE_FEED_MAX_AGE_SECONDS: u64 = 120;

// 100 basis points = 1%
pub static PYTH_CONFIDENCE_THRESHOLD_BPS: u64 = 100;

pub static AUM_DECIMALS: u8 = 9;

pub static PRECISION: u64 = 10u64.pow(AUM_DECIMALS as u32);
