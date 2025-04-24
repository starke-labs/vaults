use anchor_lang::prelude::*;

use crate::constants::STARKE_AUTHORITY;
use crate::state::{ManagerWhitelist, StarkeConfig, TokenWhitelist, TokenWhitelistError};

pub fn _initialize_starke(ctx: Context<InitializeStarke>) -> Result<()> {
    msg!("Initializing token whitelist");
    msg!("Authority: {}", ctx.accounts.authority.key());
    msg!(
        "Token whitelist account: {}",
        ctx.accounts.token_whitelist.key()
    );
    msg!("Token whitelist bump: {}", ctx.bumps.token_whitelist);
    msg!(
        "Manager whitelist account: {}",
        ctx.accounts.manager_whitelist.key()
    );
    msg!("Manager whitelist bump: {}", ctx.bumps.manager_whitelist);
    msg!(
        "Starke config account: {}",
        ctx.accounts.starke_config.key()
    );
    msg!("Starke config bump: {}", ctx.bumps.starke_config);

    let authority_key = ctx.accounts.authority.key();
    ctx.accounts.token_whitelist.initialize(
        &authority_key,
        &STARKE_AUTHORITY,
        ctx.bumps.token_whitelist,
    )?;
    msg!("Token whitelist initialized successfully");

    ctx.accounts
        .manager_whitelist
        .initialize(ctx.bumps.manager_whitelist)?;

    msg!("Manager whitelist initialized successfully");
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
        address = STARKE_AUTHORITY @ TokenWhitelistError::UnauthorizedAccess,
    )]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = TokenWhitelist::MAX_SPACE,
        seeds = [TokenWhitelist::SEED],
        bump,
    )]
    pub token_whitelist: Box<Account<'info, TokenWhitelist>>,

    #[account(
        init,
        payer = authority,
        space = ManagerWhitelist::MAX_SPACE,
        seeds = [ManagerWhitelist::SEED],
        bump,
    )]
    pub manager_whitelist: Box<Account<'info, ManagerWhitelist>>,

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
