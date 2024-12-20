use anchor_lang::prelude::*;
use anchor_spl::token::*;

use crate::state::Vault;

pub fn mint_vault_token<'info>(
    vault: Box<Account<'info, Vault>>,
    mint: Box<Account<'info, Mint>>,
    to: Box<Account<'info, TokenAccount>>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
    token_program: Program<'info, Token>,
) -> Result<()> {
    let mint_accounts = MintTo {
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: vault.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(token_program.to_account_info(), mint_accounts, signer_seeds);
    mint_to(cpi_ctx, amount)?;

    Ok(())
}

pub fn burn_vault_token<'info>(
    user: Signer<'info>,
    mint: Box<Account<'info, Mint>>,
    from: Box<Account<'info, TokenAccount>>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
    token_program: Program<'info, Token>,
) -> Result<()> {
    let burn_accounts = Burn {
        mint: mint.to_account_info(),
        from: from.to_account_info(),
        authority: user.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(token_program.to_account_info(), burn_accounts, signer_seeds);
    burn(cpi_ctx, amount)?;

    Ok(())
}
