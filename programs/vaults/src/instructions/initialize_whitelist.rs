use anchor_lang::prelude::*;

use crate::constants::PROGRAM_AUTHORITY;
use crate::state::{TokenWhitelist, WhitelistError};

pub fn _initialize_whitelist(ctx: Context<InitializeWhitelist>) -> Result<()> {
    let authority_key = ctx.accounts.authority.key();
    ctx.accounts
        .whitelist
        .initialize(authority_key, PROGRAM_AUTHORITY, ctx.bumps.whitelist)
}

#[derive(Accounts)]
pub struct InitializeWhitelist<'info> {
    #[account(
        mut,
        address = PROGRAM_AUTHORITY @ WhitelistError::UnauthorizedAccess,
    )]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = TokenWhitelist::MAX_SPACE,
        seeds = [TokenWhitelist::SEED],
        bump,
    )]
    pub whitelist: Box<Account<'info, TokenWhitelist>>,

    pub system_program: Program<'info, System>,
}
