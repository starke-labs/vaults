use anchor_lang::prelude::*;
use anchor_spl::token::*;

// Wrapper function for the transfer token instruction
pub fn transfer_token<'info>(
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    amount: u64,
    authority: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    _transfer_token(from, to, amount, authority, token_program, None)
}

// Wrapper function for the transfer token instruction with signer seeds
pub fn transfer_token_with_signer<'info>(
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    amount: u64,
    authority: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    token_program: &Program<'info, Token>,
) -> Result<()> {
    _transfer_token(
        from,
        to,
        amount,
        authority,
        token_program,
        Some(signer_seeds),
    )
}

// Actual implementation of the transfer token instruction
fn _transfer_token<'info>(
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    amount: u64,
    authority: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let transfer_accounts = Transfer {
        from: from.to_account_info().clone(),
        to: to.to_account_info().clone(),
        authority: (*authority).clone(),
    };
    let cpi_ctx: CpiContext<'_, '_, '_, 'info, Transfer<'info>>;
    match signer_seeds {
        Some(seeds) => {
            cpi_ctx = CpiContext::new_with_signer(
                token_program.to_account_info().clone(),
                transfer_accounts,
                seeds,
            );
        }
        None => {
            cpi_ctx = CpiContext::new(token_program.to_account_info().clone(), transfer_accounts);
        }
    }
    transfer(cpi_ctx, amount)?;

    Ok(())
}
