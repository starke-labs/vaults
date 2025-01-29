use anchor_lang::prelude::*;

use anchor_spl::associated_token::{create_idempotent, AssociatedToken, Create};
use anchor_spl::token::{Mint, Token};

/// Creates an associated token account for the given token mint and owner if it doesn't already exist.
pub fn create_associated_token_account<'info>(
    user: &Signer<'info>,
    mint: &Account<'info, Mint>,
    associated_token: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    associated_token_program: &Program<'info, AssociatedToken>,
) -> Result<()> {
    // Check if the associated token account already exists
    // TODO: Verify if this is the only check that will confirm the account doesn't already exist
    let create_accounts = Create {
        payer: user.to_account_info(),
        associated_token: associated_token.to_account_info(),
        authority: user.to_account_info(),
        mint: mint.to_account_info(),
        system_program: system_program.to_account_info(),
        token_program: token_program.to_account_info(),
    };
    // TODO: Check if signer needed for this tx
    let cpi_ctx = CpiContext::new(associated_token_program.to_account_info(), create_accounts);

    create_idempotent(cpi_ctx)
}
