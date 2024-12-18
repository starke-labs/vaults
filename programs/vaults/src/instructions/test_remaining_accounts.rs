use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

pub fn _test_remaining_accounts<'info>(
    ctx: Context<'_, '_, 'info, 'info, TestRemainingAccounts<'info>>,
) -> Result<()> {
    msg!(
        "Remaining accounts length: {}",
        ctx.remaining_accounts.len()
    );

    ctx.remaining_accounts
        .iter()
        .try_for_each(|a| -> Result<()> {
            msg!("Remaining account: {}", a.key());
            msg!("Remaining account is signer: {}", a.is_signer);
            msg!("Remaining account is writable: {}", a.is_writable);
            msg!("Remaining account owner: {:?}", a.owner);
            msg!("Remaining account lamports: {:?}", a.lamports);

            let token_account: Account<'info, TokenAccount> = Account::try_from(a)?;
            msg!("Remaining account mint: {:?}", token_account.mint);

            Ok(())
        })?;

    Ok(())
}

#[derive(Accounts)]
pub struct TestRemainingAccounts<'info> {
    pub signer: Signer<'info>,
}
