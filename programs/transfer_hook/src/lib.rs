use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
// use vaults::state::Vault;

pub mod constants;
pub mod state;

use constants::EXTRA_ACCOUNT_META_SEED;
use state::VaultConfigs;

declare_id!("3Mbtr8yzqLUuBZVSefrVtAPmgNLFutEXeRWJNATsKU5z");

#[error_code]
pub enum TransferHookError {
    #[msg("The token is currently non-transferrable")]
    TokenNonTransferrable,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Mint does not match vault")]
    MintDoesNotMatchVault,
}

#[program]
pub mod transfer_hook {
    use super::*;

    pub fn initialize_extra_account_metas(
        ctx: Context<InitializeExtraAccountMetaAccounts>,
    ) -> Result<()> {
        let vault_configs = &mut ctx.accounts.vault_configs;
        vault_configs.initialize(ctx.bumps.vault_configs)?;

        let extra_account_metas = InitializeExtraAccountMetaAccounts::extra_account_metas()?;
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        Ok(())
    }

    // #[instruction(discriminator = b"spl-transfer-hook-interface:initialize-extra-account-metas")]
    pub fn add_vault_config(
        ctx: Context<InitializeExtraAccountMetaAccounts>,
        vtoken_is_transferrable: bool,
    ) -> Result<()> {
        msg!("add_vault_config {}", vtoken_is_transferrable);

        let vault_configs = &mut ctx.accounts.vault_configs;
        vault_configs.add_vault_config(
            // &ctx.accounts.vault.key(),
            &pubkey!("3Mbtr8yzqLUuBZVSefrVtAPmgNLFutEXeRWJNATsKU5z"),
            &ctx.accounts.manager.key(),
            &ctx.accounts.mint.key(),
            vtoken_is_transferrable,
            ctx.bumps.vault_configs,
        )?;

        let extra_account_metas = InitializeExtraAccountMetaAccounts::extra_account_metas()?;
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        Ok(())
    }

    #[instruction(discriminator = b"spl-transfer-hook-interface:execute")]
    pub fn execute(ctx: Context<ExecuteAccounts>, _amount: u64) -> Result<()> {
        let vault_configs = &mut ctx.accounts.vault_configs;
        let vtoken_is_transferrable =
            vault_configs.check_if_vtoken_is_transferrable(&ctx.accounts.mint.key())?;
        require!(
            vtoken_is_transferrable,
            TransferHookError::TokenNonTransferrable
        );

        Ok(())
    }

    pub fn set_vtoken_is_transferrable(
        ctx: Context<SetVtokenIsTransferrableAccounts>,
        vtoken_is_transferrable: bool,
    ) -> Result<()> {
        let vault_configs = &mut ctx.accounts.vault_configs;
        vault_configs
            .set_vtoken_is_transferrable(&ctx.accounts.mint.key(), vtoken_is_transferrable)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaAccounts<'info> {
    // Payer is the manager of the vault
    #[account(mut)]
    manager: Signer<'info>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        init_if_needed,
        seeds = [EXTRA_ACCOUNT_META_SEED, mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            InitializeExtraAccountMetaAccounts::extra_account_metas()?.len()
        )?,
        payer = manager
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    #[account(
        init,
        seeds = [VaultConfigs::SEED, mint.key().as_ref()],
        bump,
        payer = manager,
        space = VaultConfigs::MAX_SPACE
    )]
    pub vault_configs: Account<'info, VaultConfigs>,

    pub system_program: Program<'info, System>,
}

// Define extra account metas to store on extra_account_meta_list account
impl<'info> InitializeExtraAccountMetaAccounts<'info> {
    pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
        msg!("extra_account_metas");
        Ok(vec![ExtraAccountMeta::new_with_seeds(
            &[Seed::Literal {
                bytes: VaultConfigs::SEED.to_vec(),
            }],
            false, // is_signer
            true,  // is_writable
        )?])
    }
}

#[derive(Accounts)]
pub struct AddVaultConfigAccounts<'info> {
    #[account(mut)]
    manager: Signer<'info>,

    // #[account(
    //     constraint = vault.manager == manager.key() @ TransferHookError::Unauthorized,
    // )]
    // pub vault: Account<'info, Vault>,
    pub mint: InterfaceAccount<'info, Mint>,
}

// Order of accounts matters for this struct
// The first 4 accounts are the accounts required for token transfer (source, mint, destination, owner)
// Remaining accounts are the extra accounts required from the ExtraAccountMetaList account
// These accounts are provided via CPI to this program from the Token2022 program
#[derive(Accounts)]
pub struct ExecuteAccounts<'info> {
    #[account(token::mint = mint, token::authority = owner)]
    pub source_token: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(token::mint = mint)]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Source token account owner, can be SystemAccount or PDA owned by another program
    pub owner: UncheckedAccount<'info>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(seeds = [EXTRA_ACCOUNT_META_SEED, mint.key().as_ref()], bump)]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    #[account(seeds = [VaultConfigs::SEED], bump = vault_configs.bump)]
    pub vault_configs: Account<'info, VaultConfigs>,
}

#[derive(Accounts)]
pub struct SetVtokenIsTransferrableAccounts<'info> {
    #[account(mut)]
    manager: Signer<'info>,

    // #[account(
    //     constraint = vault.manager == manager.key() @ TransferHookError::Unauthorized,
    // )]
    // pub vault: Account<'info, Vault>,
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(mut, seeds = [VaultConfigs::SEED, mint.key().as_ref()], bump = vault_configs.bump)]
    vault_configs: Account<'info, VaultConfigs>,
}
