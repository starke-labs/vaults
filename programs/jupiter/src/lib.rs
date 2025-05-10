#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

// NOTE: Running anchor keys sync updates this ID, which is not
//       correct as this needs to track the Jupiter mainnet program ID
declare_id!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");

#[program]
pub mod jupiter {}
