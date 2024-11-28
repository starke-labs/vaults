use anchor_lang::prelude::*;

// TODO: What happens when we need to redeploy the program?
pub static PROGRAM_AUTHORITY: Pubkey = pubkey!("26jdGTuEWEP5PaZa9PqJ4Q1i1Zj9ct51T8bBqhcN2ZTf");

pub static TOKEN_WHITELIST_SEED: &[u8] = b"STARKE_TOKEN_WHITELIST";
