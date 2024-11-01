mod depositor;
mod event;
mod vault;

use anchor_lang::prelude::*;
use anchor_spl::token::*;

use depositor::*;
use event::*;
use vault::*;

declare_id!("BvaZoX158jbctxEe5jKanBCQngEUkzz8KSAVTD4mvkTF");

#[program]
pub mod vault_manager {
    use super::*;

    pub fn create_vault(ctx: Context<CreateVault>, name: String) -> Result<()> {
        ctx.accounts.vault.initialize(
            *ctx.accounts.manager.key,
            ctx.accounts.deposit_token.key(),
            name,
            ctx.bumps.vault,
        )?;

        emit!(VaultCreated {
            vault: ctx.accounts.vault.key(),
            manager: *ctx.accounts.manager.key,
            deposit_token: ctx.accounts.deposit_token.key(),
            name: ctx.accounts.vault.name.to_string(),
            timestamp: ctx.accounts.clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // Transfer tokens from depositor to vault
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.depositor_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.depositor.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)?;

        // Update depositor account
        let depositor_account = &mut ctx.accounts.depositor_account;
        if depositor_account.amount == 0 {
            depositor_account.initialize(
                ctx.accounts.vault.key(),
                ctx.accounts.depositor.key(),
                ctx.bumps.depositor_account,
            );
        }
        depositor_account.deposit(amount)?;

        emit!(DepositMade {
            vault: ctx.accounts.vault.key(),
            depositor: ctx.accounts.depositor.key(),
            amount,
            total_deposited: depositor_account.amount,
            timestamp: ctx.accounts.clock.unix_timestamp,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    // Vault manager
    #[account(mut)]
    pub manager: Signer<'info>,

    // Vault (PDA)
    // Currently seeded by manager pubkey, which means that the manager can only create one vault
    // This also means when users want to deposit, they need to send manager's pubkey in accounts
    // Do we want this? Or should we avoid PDA here and store vault pubkeys instead?
    // TODO: Find a way to allow multiple vaults per manager for v1
    #[account(
        init,
        payer = manager,
        space = Vault::MAX_SPACE,
        seeds = [b"vault", manager.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    pub deposit_token: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

// TODO: Error when running `anchor test`
//       `Error: Reached maximum depth for account resolution`
#[derive(Accounts)]
pub struct Deposit<'info> {
    // Depositor
    #[account(mut)]
    pub depositor: Signer<'info>,

    // Depositor's token account
    #[account(
        mut,
        constraint = depositor_token_account.owner == depositor.key(),
        constraint = depositor_token_account.mint == vault.deposit_token,
    )]
    pub depositor_token_account: Account<'info, TokenAccount>,

    // Manager
    /// CHECK: We don't need to check the manager's key
    pub manager: UncheckedAccount<'info>,

    // Vault
    // `vault.manager.key().as_ref()` throws "Error: Reached maximum depth for account resolution"
    // `manager.key().as_ref()` works
    #[account(
        mut,
        seeds = [b"vault", manager.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    // Vault's token account
    #[account(
        mut,
        constraint = vault_token_account.owner == manager.key(),
        constraint = vault_token_account.mint == vault.deposit_token,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    // Depositor account (PDA)
    // Currently seeded by vault pubkey and depositor pubkey, which means that each depositor can deposit to multiple vaults
    // TODO: For v1, we should allow only one depositor per vault
    #[account(
        init_if_needed,
        payer = depositor,
        space = Depositor::MAX_SPACE,
        seeds = [b"depositor", vault.key().as_ref(), depositor.key().as_ref()],
        bump
    )]
    pub depositor_account: Account<'info, Depositor>,

    // Token program is required for `transfer_checked`
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}
