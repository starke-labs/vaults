use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};

use crate::jupiter::Jupiter;
use crate::state::Vault;

pub fn _swap_on_jupiter(ctx: Context<SwapOnJupiter>, data: Vec<u8>) -> Result<()> {
    let accounts: Vec<AccountMeta> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| AccountMeta {
            pubkey: *acc.key,
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let accounts_infos: Vec<AccountInfo> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    let manager = ctx.accounts.manager.key();
    let vault_seeds = &[Vault::SEED, manager.as_ref(), &[ctx.accounts.vault.bump]];
    let signer_seeds = &[&vault_seeds[..]];

    invoke_signed(
        &Instruction {
            program_id: *ctx.accounts.jupiter_program.key,
            accounts,
            data,
        },
        &accounts_infos,
        signer_seeds,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct SwapOnJupiter<'info> {
    // Manager
    /// CHECK: We can skip checking the manager
    pub manager: UncheckedAccount<'info>,

    // Vault
    #[account(
        seeds = [Vault::SEED, manager.key().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub jupiter_program: Program<'info, Jupiter>,
}
