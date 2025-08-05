use anchor_lang::prelude::*;

use crate::constants::STARKE_AUTHORITY;
use crate::state::{
    InvestorType, StarkeConfig, StarkeConfigError, UserWhitelist, UserWhitelistError,
};

pub fn _add_user(
    ctx: Context<ModifyUserWhitelist>,
    user: Pubkey,
    investor_type: InvestorType,
) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    msg!("Adding user to whitelist: {}", user);
    msg!("Investor type: {:?}", investor_type);

    ctx.accounts.user_whitelist.add_user(user, investor_type)?;

    msg!("Successfully added user to whitelist");
    Ok(())
}

pub fn _remove_user(ctx: Context<ModifyUserWhitelist>, user: Pubkey) -> Result<()> {
    require!(
        !ctx.accounts.starke_config.is_paused,
        StarkeConfigError::StarkePaused
    );

    msg!("Removing user from whitelist: {}", user);

    ctx.accounts.user_whitelist.remove_user(user)?;

    msg!("Successfully removed user from whitelist");
    Ok(())
}

#[derive(Accounts)]
pub struct ModifyUserWhitelist<'info> {
    #[account(mut, address = STARKE_AUTHORITY @ UserWhitelistError::UnauthorizedAccess)]
    pub starke_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [UserWhitelist::SEED],
        bump = user_whitelist.bump,
    )]
    pub user_whitelist: Box<Account<'info, UserWhitelist>>,

    #[account(
        seeds = [StarkeConfig::SEED],
        bump = starke_config.bump,
    )]
    pub starke_config: Box<Account<'info, StarkeConfig>>,

    pub system_program: Program<'info, System>,
}
