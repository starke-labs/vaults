mod event;
mod vault;
mod vault_balance;
mod whitelist;

use anchor_lang::prelude::*;
use anchor_spl::token::*;

use event::*;
use vault::*;
use vault_balance::*;
use whitelist::*;

declare_id!("5bZpuR8pbHpndAcJ99Hc3fNYT65fagRCptFvCwqPw3Te");

// TODO: What happens when we need to redeploy the program?
pub const PROGRAM_AUTHORITY: Pubkey = pubkey!("26jdGTuEWEP5PaZa9PqJ4Q1i1Zj9ct51T8bBqhcN2ZTf");
pub const TOKEN_WHITELIST_SEED: &[u8] = b"STARKE_TOKEN_WHITELIST";

#[program]
pub mod vault_manager {
    use super::*;

    pub fn initialize_whitelist(ctx: Context<InitializeWhitelist>) -> Result<()> {
        ctx.accounts.whitelist.initialize(
            ctx.accounts.authority.key(),
            PROGRAM_AUTHORITY,
            ctx.bumps.whitelist,
        )
    }

    pub fn add_token(ctx: Context<ModifyWhitelist>, token: Pubkey) -> Result<()> {
        ctx.accounts.whitelist.add_token(token)?;

        emit!(TokenWhitelisted {
            token,
            timestamp: ctx.accounts.clock.unix_timestamp,
        });

        Ok(())
    }

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

    pub fn deposit(ctx: Context<DepositOrWithdraw>, amount: u64) -> Result<()> {
        // Transfer tokens from depositor to vault
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)?;

        // Update vault total deposits
        ctx.accounts.vault.deposit(amount)?;

        // Update depositor account
        let vault_balance = &mut ctx.accounts.vault_balance;
        if vault_balance.amount == 0 {
            vault_balance.initialize(
                ctx.accounts.vault.key(),
                ctx.accounts.user.key(),
                ctx.bumps.vault_balance,
            );
        }
        vault_balance.deposit(amount)?;

        emit!(DepositMade {
            vault: ctx.accounts.vault.key(),
            user: ctx.accounts.user.key(),
            amount,
            total_deposited: vault_balance.amount,
            timestamp: ctx.accounts.clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn withdraw(ctx: Context<DepositOrWithdraw>, amount: u64) -> Result<()> {
        // Update vault total deposits
        ctx.accounts.vault.withdraw(amount)?;

        // Update depositor account
        let vault_balance = &mut ctx.accounts.vault_balance;
        vault_balance.withdraw(amount)?;

        // Close the depositor account if balance is 0 and return rent to depositor
        if vault_balance.amount == 0 {
            let vault_balance_lamports = vault_balance.to_account_info().lamports();
            let dest_starting_lamports = ctx.accounts.user.lamports();

            **vault_balance.to_account_info().try_borrow_mut_lamports()? = 0;
            **ctx.accounts.user.try_borrow_mut_lamports()? = dest_starting_lamports
                .checked_add(vault_balance_lamports)
                .ok_or(VaultBalanceError::NumericOverflow)?;
        }

        // Transfer tokens from vault to depositor using PDA signer
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.vault.to_account_info(),
        };
        let manager_key = ctx.accounts.manager.key();
        let seeds = &[
            Vault::SEED,
            manager_key.as_ref(),
            &[ctx.accounts.vault.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer(cpi_ctx, amount)?;

        emit!(WithdrawMade {
            vault: ctx.accounts.vault.key(),
            user: ctx.accounts.user.key(),
            amount,
            remaining_balance: vault_balance.amount,
            timestamp: ctx.accounts.clock.unix_timestamp,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeWhitelist<'info> {
    #[account(
        mut,
        address = PROGRAM_AUTHORITY @ WhitelistError::UnauthorizedAccess,
    )]
    pub authority: Signer<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        space = TokenWhitelist::MAX_SPACE,
        seeds = [TOKEN_WHITELIST_SEED],
        bump,
    )]
    pub whitelist: Account<'info, TokenWhitelist>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ModifyWhitelist<'info> {
    #[account(
        address = PROGRAM_AUTHORITY @ WhitelistError::UnauthorizedAccess,
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [TOKEN_WHITELIST_SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Account<'info, TokenWhitelist>,

    pub clock: Sysvar<'info, Clock>,
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
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        seeds = [TOKEN_WHITELIST_SEED],
        bump = whitelist.bump,
    )]
    pub whitelist: Account<'info, TokenWhitelist>,

    #[account(
        constraint = whitelist.is_whitelisted(&deposit_token.key()) @ WhitelistError::TokenNotWhitelisted,
    )]
    pub deposit_token: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct DepositOrWithdraw<'info> {
    // Depositor
    #[account(mut)]
    pub user: Signer<'info>,

    // Depositor's token account
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == vault.deposit_token,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    // Manager
    /// CHECK: We don't need to check the manager's key
    pub manager: UncheckedAccount<'info>,

    // Vault
    // `vault.manager.key().as_ref()` throws "Error: Reached maximum depth for account resolution"
    // `manager.key().as_ref()` works
    #[account(
        mut,
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    // TODO: We should get it from the vault account
    // Vault's token account
    #[account(
        mut,
        constraint = vault_token_account.owner == vault.key(),
        constraint = vault_token_account.mint == vault.deposit_token,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    // Depositor account (PDA)
    // Currently seeded by vault pubkey and depositor pubkey, which means that each depositor can deposit to multiple vaults
    // TODO: For v1, we should allow only one depositor per vault
    #[account(
        init_if_needed,
        payer = user,
        space = VaultBalance::MAX_SPACE,
        seeds = [VaultBalance::SEED, vault.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub vault_balance: Account<'info, VaultBalance>,

    // Token program is required for `transfer_checked`
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}
