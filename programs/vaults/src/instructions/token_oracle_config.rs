use anchor_lang::prelude::*;

use crate::{
    constants::{PYTH_CONFIDENCE_THRESHOLD_BPS, PYTH_PRICE_FEED_MAX_AGE_SECONDS, STARKE_AUTHORITY},
    state::{TokenOracleConfig, TokenOracleConfigUpdated, TokenWhitelist, TokenWhitelistError},
};

pub fn _set_token_pyth_pro_oracle(
    ctx: Context<SetTokenPythProOracle>,
    mint: Pubkey,
    price_feed_id: u32,
    channel_id: u8,
    max_age_seconds: Option<u64>,
    confidence_threshold_bps: Option<u64>,
    is_active: bool,
) -> Result<()> {
    require!(
        ctx.accounts.token_whitelist.is_whitelisted(&mint),
        TokenWhitelistError::TokenNotWhitelisted
    );

    let max_age_seconds = max_age_seconds.unwrap_or(PYTH_PRICE_FEED_MAX_AGE_SECONDS);
    let confidence_threshold_bps =
        confidence_threshold_bps.unwrap_or(PYTH_CONFIDENCE_THRESHOLD_BPS);
    ctx.accounts.token_oracle_config.set_pyth_pro(
        ctx.accounts.authority.key(),
        mint,
        price_feed_id,
        channel_id,
        max_age_seconds,
        confidence_threshold_bps,
        is_active,
        ctx.accounts.clock.unix_timestamp,
        ctx.bumps.token_oracle_config,
    )?;

    emit!(TokenOracleConfigUpdated {
        mint,
        price_feed_id,
        channel_id,
        max_age_seconds,
        confidence_threshold_bps,
        is_active,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(mint: Pubkey)]
pub struct SetTokenPythProOracle<'info> {
    #[account(
        mut,
        address = STARKE_AUTHORITY @ TokenWhitelistError::UnauthorizedAccess,
    )]
    pub authority: Signer<'info>,

    #[account(
        seeds = [TokenWhitelist::SEED],
        bump = token_whitelist.bump,
    )]
    pub token_whitelist: Box<Account<'info, TokenWhitelist>>,

    #[account(
        init_if_needed,
        payer = authority,
        space = TokenOracleConfig::MAX_SPACE,
        seeds = [TokenOracleConfig::SEED, mint.as_ref()],
        bump,
    )]
    pub token_oracle_config: Box<Account<'info, TokenOracleConfig>>,

    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}
