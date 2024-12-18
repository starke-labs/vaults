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
            let token_account: Account<'info, TokenAccount> = Account::try_from(a)?;
            msg!("Remaining account mint: {:?}", token_account.mint);
            msg!("Remaining account balance: {:?}", token_account.amount);

            Ok(())
        })?;

    Ok(())
}

#[derive(Accounts)]
pub struct TestRemainingAccounts<'info> {
    pub signer: Signer<'info>,
}
