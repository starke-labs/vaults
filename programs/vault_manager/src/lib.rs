use anchor_lang::prelude::*;

mod vault;
use vault::*;

declare_id!("BvaZoX158jbctxEe5jKanBCQngEUkzz8KSAVTD4mvkTF");

#[program]
pub mod vault_manager {
    use super::*;

    pub fn create_vault(
        ctx: Context<CreateVault>,
        deposit_token: Pubkey,
        name: String,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        // TODO: does the manager need to be whitelisted?
        vault.manager = *ctx.accounts.manager.key;
        vault.bump = ctx.bumps.vault;
        // TODO: does the deposit token need to be whitelisted?
        vault.deposit_token = deposit_token;
        vault.name = name;

        // TODO: figure out logs organisation
        msg!("Vault created");
        msg!("Vault name: {}", vault.name);
        msg!("Vault manager: {}", vault.manager);
        msg!("Vault deposit token: {}", vault.deposit_token);

        Ok(())
    }

    pub fn update_vault(
        ctx: Context<UpdateVault>,
        name: Option<String>,
        deposit_token: Option<Pubkey>,
    ) -> Result<()> {
        ctx.accounts.vault.update(name, deposit_token)?;
        msg!("Vault updated");

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub manager: Signer<'info>,

    // currently seeded by manager pubkey, which means that the manager can only create one vault
    // TODO: find a way to allow multiple vaults per manager for v1
    // space: 8 (discriminator) + 32 (manager pubkey) + 32 (deposit token pubkey) + 4 (name length) + 200 (name) + 1 (bump)
    #[account(
        init,
        payer = manager,
        space = 8 + 32 + 32 + 4 + 200 + 1,
        seeds = [b"vault", manager.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateVault<'info> {
    pub manager: Signer<'info>,

    #[account(mut, seeds = [b"vault", manager.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
}
