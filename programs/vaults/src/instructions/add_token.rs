use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::constants::PROGRAM_AUTHORITY;
use crate::state::{TokenWhitelist, TokenWhitelisted, WhitelistError};

pub fn _add_token(ctx: Context<ModifyWhitelist>, price_feed_id: &str) -> Result<()> {
    ctx.accounts.whitelist.add_token(
        ctx.accounts.token_mint.key(),
        price_feed_id,
        ctx.accounts.price_update.key(),
    )?;

    emit!(TokenWhitelisted {
        token: ctx.accounts.token_mint.key(),
        price_feed_id: price_feed_id.to_string(),
        price_update: ctx.accounts.price_update.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct ModifyWhitelist<'info> {
    #[account(
        address = PROGRAM_AUTHORITY @ WhitelistError::UnauthorizedAccess,
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [TokenWhitelist::SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Box<Account<'info, TokenWhitelist>>,

    pub token_mint: Box<Account<'info, Mint>>,
    pub price_update: Box<Account<'info, PriceUpdateV2>>,
    pub clock: Sysvar<'info, Clock>,
}
