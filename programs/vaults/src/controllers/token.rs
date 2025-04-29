use anchor_lang::prelude::*;
use anchor_spl::token_interface::*;

/// Function for the transfer token instruction
pub fn transfer_token<'info>(
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &AccountInfo<'info>,
    token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    _transfer_token(from, to, amount, mint, authority, token_program, None)
}

/// Function for the transfer token instruction with signer seeds
pub fn transfer_token_with_signer<'info>(
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    _transfer_token(
        from,
        to,
        amount,
        mint,
        authority,
        token_program,
        Some(signer_seeds),
    )
}

/// Actual implementation of the transfer token instruction
fn _transfer_token<'info>(
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &AccountInfo<'info>,
    token_program: &Interface<'info, TokenInterface>,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let transfer_accounts = TransferChecked {
        from: from.to_account_info().clone(),
        mint: mint.to_account_info().clone(),
        to: to.to_account_info().clone(),
        authority: (*authority).clone(),
    };
    let cpi_ctx: CpiContext<'_, '_, '_, 'info, TransferChecked<'info>>;
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
    transfer_checked(cpi_ctx, amount, mint.decimals)?;

    Ok(())
}
