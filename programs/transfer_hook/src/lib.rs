use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use vaults::state::Vault;

pub mod constants;
pub mod state;

use constants::EXTRA_ACCOUNT_META_SEED;
use state::VaultConfig;

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

    // #[instruction(discriminator = b"spl-transfer-hook-interface:initialize-extra-account-metas")]
    pub fn initialize_extra_account_metas(
        ctx: Context<InitializeExtraAccountMetasAccounts>,
        vtoken_is_transferrable: bool,
    ) -> Result<()> {
        let vault_config = &mut ctx.accounts.vault_config;
        vault_config.initialize(
            &ctx.accounts.vault.key(),
            &ctx.accounts.vault.manager,
            &ctx.accounts.vault.mint.key(),
            vtoken_is_transferrable,
            ctx.bumps.vault_config,
        )?;

        let extra_account_metas = InitializeExtraAccountMetasAccounts::extra_account_metas(
            &ctx.accounts.vault.mint.key(),
        )?;
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        Ok(())
    }

    #[instruction(discriminator = b"spl-transfer-hook-interface:execute")]
    pub fn execute(ctx: Context<ExecuteAccounts>, _amount: u64) -> Result<()> {
        let vault_config = &ctx.accounts.vault_config;
        let vtoken_is_transferrable = vault_config.check_if_vtoken_is_transferrable()?;
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
        let vault_config = &mut ctx.accounts.vault_config;
        vault_config.set_vtoken_is_transferrable(vtoken_is_transferrable)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeExtraAccountMetasAccounts<'info> {
    // Payer is the manager of the vault
    #[account(mut)]
    manager: Signer<'info>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        init,
        seeds = [EXTRA_ACCOUNT_META_SEED, vault.mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            InitializeExtraAccountMetasAccounts::extra_account_metas(&vault.mint.key())?.len()
        )?,
        payer = manager
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    #[account(
        constraint = vault.manager == manager.key() @ TransferHookError::Unauthorized,
    )]
    pub vault: Account<'info, Vault>,

    #[account(init_if_needed, seeds = [VaultConfig::SEED, vault.mint.key().as_ref()], bump, payer = manager, space = VaultConfig::MAX_SPACE)]
    pub vault_config: Account<'info, VaultConfig>,

    pub system_program: Program<'info, System>,
}

// Define extra account metas to store on extra_account_meta_list account
impl<'info> InitializeExtraAccountMetasAccounts<'info> {
    pub fn extra_account_metas(mint: &Pubkey) -> Result<Vec<ExtraAccountMeta>> {
        Ok(vec![ExtraAccountMeta::new_with_seeds(
            &[Seed::Literal {
                bytes: [VaultConfig::SEED.to_vec(), mint.as_ref().to_vec()].concat(),
            }],
            false, // is_signer
            true,  // is_writable
        )?])
    }
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

    #[account(seeds = [VaultConfig::SEED, mint.key().as_ref()], bump = vault_config.bump)]
    pub vault_config: Account<'info, VaultConfig>,
}

#[derive(Accounts)]
pub struct SetVtokenIsTransferrableAccounts<'info> {
    #[account(mut)]
    manager: Signer<'info>,

    #[account(
        constraint = vault.manager == manager.key() @ TransferHookError::Unauthorized,
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut, seeds = [VaultConfig::SEED, vault.mint.key().as_ref()], bump = vault_config.bump)]
    vault_config: Account<'info, VaultConfig>,
}
