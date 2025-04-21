use anchor_lang::prelude::*;

use crate::constants::STARKE_AUTHORITY;
use crate::state::{StarkeConfig, TokenWhitelist, WhitelistError};

pub fn _initialize_starke(ctx: Context<InitializeStarke>) -> Result<()> {
    msg!("Initializing token whitelist");
    msg!("Authority: {}", ctx.accounts.authority.key());
    msg!("Whitelist account: {}", ctx.accounts.whitelist.key());
    msg!("Whitelist bump: {}", ctx.bumps.whitelist);
    msg!(
        "Starke config account: {}",
        ctx.accounts.starke_config.key()
    );
    msg!("Starke config bump: {}", ctx.bumps.starke_config);

    let authority_key = ctx.accounts.authority.key();
    ctx.accounts
        .whitelist
        .initialize(&authority_key, &STARKE_AUTHORITY, ctx.bumps.whitelist)?;
    msg!("Whitelist initialized successfully");

    ctx.accounts
        .starke_config
        .initialize(ctx.bumps.starke_config)?;
    msg!("Starke config initialized successfully");

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeStarke<'info> {
    #[account(
        mut,
        address = STARKE_AUTHORITY @ WhitelistError::UnauthorizedAccess,
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

    #[account(
        init,
        payer = authority,
        space = StarkeConfig::MAX_SPACE,
        seeds = [StarkeConfig::SEED],
        bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    pub system_program: Program<'info, System>,
}
