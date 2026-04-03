use crate::{
    constants::STARKE_AUTHORITY,
    state::{TokenWhitelist, TokenWhitelistError, WhitelistTokenAdded, WhitelistTokenRemoved},
};
use anchor_lang::prelude::*;

pub fn _add_token(
    ctx: Context<ModifyTokenWhitelist>,
    token: Pubkey,
    price_feed_id: String,
    price_update: Pubkey,
) -> Result<()> {
    msg!("Processing request to add token: {}", token);
    msg!("Price feed ID: {}", price_feed_id);
    msg!("Price update pubkey: {}", price_update);

    ctx.accounts
        .token_whitelist
        .add_token(token, price_feed_id.clone(), price_update)?;

    msg!("Successfully added token to whitelist");

    emit!(WhitelistTokenAdded {
        mint: token,
        price_feed_id,
        price_update,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

pub fn _remove_token(ctx: Context<ModifyTokenWhitelist>, token: Pubkey) -> Result<()> {
    msg!("Processing request to remove token: {}", token);

    ctx.accounts.token_whitelist.remove_token(&token)?;

    msg!("Successfully removed token from whitelist");

    emit!(WhitelistTokenRemoved {
        mint: token,
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts, AnchorSerialize, AnchorDeserialize)]
pub struct ModifyTokenWhitelist<'info> {
    #[account(
        address = STARKE_AUTHORITY @ TokenWhitelistError::UnauthorizedAccess,
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [TokenWhitelist::SEED],
        bump = token_whitelist.bump,
    )]
    pub token_whitelist: Box<Account<'info, TokenWhitelist>>,

    pub clock: Sysvar<'info, Clock>,
}
