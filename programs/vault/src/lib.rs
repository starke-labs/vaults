use anchor_lang::prelude::*;
// use anchor_spl::token::{self, Burn, Mint, MintTo, TokenAccount, Transfer};

declare_id!("BvaZoX158jbctxEe5jKanBCQngEUkzz8KSAVTD4mvkTF");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, deposit_token: Pubkey) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.manager = *ctx.accounts.vault_owner.key;
        vault.deposit_token = deposit_token;

        msg!("Vault initialized with manager: {:?}", vault.manager);
        msg!("Deposit token: {:?}", vault.deposit_token);
        Ok(())
    }

    // pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    //     // Transfer tokens from user to vault
    //     let cpi_accounts = Transfer {
    //         from: ctx.accounts.user_token_account.to_account_info(),
    //         to: ctx.accounts.vault_token_account.to_account_info(),
    //         authority: ctx.accounts.user.to_account_info(),
    //     };
    //     let cpi_program = ctx.accounts.token_program.to_account_info();
    //     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    //     token::transfer(cpi_ctx, amount)?;

    //     // Mint share tokens to user
    //     let cpi_accounts = MintTo {
    //         mint: ctx.accounts.share_mint.to_account_info(),
    //         to: ctx.accounts.user_share_account.to_account_info(),
    //         authority: ctx.accounts.vault.to_account_info(),
    //     };
    //     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    //     token::mint_to(cpi_ctx, amount)?;

    //     Ok(())
    // }

    // pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    //     // Burn user's share tokens
    //     let cpi_accounts = Burn {
    //         mint: ctx.accounts.share_mint.to_account_info(),
    //         from: ctx.accounts.user_share_account.to_account_info(),
    //         authority: ctx.accounts.user.to_account_info(),
    //     };
    //     let cpi_program = ctx.accounts.token_program.to_account_info();
    //     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    //     token::burn(cpi_ctx, amount)?;

    //     // Transfer tokens from vault to user
    //     let cpi_accounts = Transfer {
    //         from: ctx.accounts.vault_token_account.to_account_info(),
    //         to: ctx.accounts.user_token_account.to_account_info(),
    //         authority: ctx.accounts.vault.to_account_info(),
    //     };
    //     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    //     token::transfer(cpi_ctx, amount)?;

    //     Ok(())
    // }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = vault_owner,
        space = 8 + 32 + 32  // discriminator + owner pubkey + deposit token mint pubkey
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub vault_owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Vault {
    pub manager: Pubkey,
    pub deposit_token: Pubkey,
}

// #[derive(Accounts)]
// pub struct Deposit<'info> {
//     #[account(mut)]
//     pub user: Signer<'info>,
//     #[account(
//         mut,
//         constraint = user_token_account.mint == vault.deposit_token,
//         constraint = user_token_account.owner == user.key()
//     )]
//     pub user_token_account: Account<'info, TokenAccount>,
//     #[account(
//         mut,
//         constraint = vault_token_account.mint == vault.deposit_token
//     )]
//     pub vault_token_account: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub share_mint: Account<'info, Mint>,
//     #[account(mut)]
//     pub user_share_account: Account<'info, TokenAccount>,
//     pub token_program: Program<'info, Token>,
//     #[account(has_one = owner)]
//     pub vault: Account<'info, Vault>,
// }

// #[derive(Accounts)]
// pub struct Withdraw<'info> {
//     #[account(mut)]
//     pub user: Signer<'info>,
//     #[account(
//         mut,
//         constraint = user_token_account.mint == vault.deposit_token,
//         constraint = user_token_account.owner == user.key()
//     )]
//     pub user_token_account: Account<'info, TokenAccount>,
//     #[account(
//         mut,
//         constraint = vault_token_account.mint == vault.deposit_token
//     )]
//     pub vault_token_account: Account<'info, TokenAccount>,
//     #[account(mut)]
//     pub share_mint: Account<'info, Mint>,
//     #[account(mut)]
//     pub user_share_account: Account<'info, TokenAccount>,
//     pub token_program: Program<'info, Token>,
//     #[account(has_one = owner)]
//     pub vault: Account<'info, Vault>,
// }
