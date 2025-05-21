#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};
use spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList};
use spl_transfer_hook_interface::instruction::{ExecuteInstruction, TransferHookInstruction};

pub mod constants;
pub mod state;

use constants::EXTRA_ACCOUNT_METAS_SEED;
use state::VtokenConfig;

declare_id!("Gk7syLzEbk46Ez6Fr9pApPPhTJMDavKxiN9JHAtfhZCz");

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
        ctx: Context<InitializeExtraAccountMetasAccounts>,
        vtoken_is_transferrable: bool,
    ) -> Result<()> {
        let vtoken_config = &mut ctx.accounts.vtoken_config;
        vtoken_config.initialize(
            &ctx.accounts.manager.key(),
            &ctx.accounts.mint.key(),
            vtoken_is_transferrable,
            ctx.bumps.vtoken_config,
        )?;

        let extra_account_metas = InitializeExtraAccountMetasAccounts::extra_account_metas(
            &ctx.accounts.vtoken_config.key(),
        )?;
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut ctx.accounts.extra_account_metas.try_borrow_mut_data()?,
            &extra_account_metas,
        )?;

        Ok(())
    }

    pub fn execute(ctx: Context<ExecuteAccounts>, _amount: u64) -> Result<()> {
        let vault_config = &ctx.accounts.vtoken_config;
        let vtoken_is_transferrable = vault_config.check_if_vtoken_is_transferrable()?;
        require!(
            vtoken_is_transferrable,
            TransferHookError::TokenNonTransferrable
        );

        Ok(())
    }

    /// Fallback instruction handler as workaround to anchor instruction discriminator check
    pub fn fallback<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8],
    ) -> Result<()> {
        let instruction = TransferHookInstruction::unpack(data)?;
        // Match instruction discriminator to transfer hook interface execute instruction
        // Token2022 program CPIs this instruction on token transfer
        match instruction {
            TransferHookInstruction::Execute { amount } => {
                let amount_bytes = amount.to_le_bytes();
                // Invoke custom transfer hook instruction on our program
                __private::__global::execute(program_id, accounts, &amount_bytes)
            }
            _ => return Err(ProgramError::InvalidInstructionData.into()),
        }
    }

    pub fn set_vtoken_is_transferrable(
        ctx: Context<SetVtokenIsTransferrableAccounts>,
        vtoken_is_transferrable: bool,
    ) -> Result<()> {
        let vault_config = &mut ctx.accounts.vtoken_config;
        vault_config.set_vtoken_is_transferrable(vtoken_is_transferrable)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeExtraAccountMetasAccounts<'info> {
    #[account(mut)]
    pub manager: Signer<'info>,

    /// CHECK: This account is initialized in the initialize_extra_account_metas instruction
    /// and its data is structured according to spl_tlv_account_resolution::state::ExtraAccountMetaList.
    /// The Token2022 program validates this account during CPI.
    #[account(
        init,
        seeds = [EXTRA_ACCOUNT_METAS_SEED, mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            InitializeExtraAccountMetasAccounts::extra_account_metas(&vtoken_config.key())?.len()
        )?,
        payer = manager
    )]
    pub extra_account_metas: AccountInfo<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        seeds = [VtokenConfig::SEED, mint.key().as_ref()],
        bump,
        payer = manager,
        space = VtokenConfig::MAX_SPACE
    )]
    pub vtoken_config: Account<'info, VtokenConfig>,

    pub system_program: Program<'info, System>,
}

// Define extra account metas to store on extra_account_meta_list account
impl<'info> InitializeExtraAccountMetasAccounts<'info> {
    pub fn extra_account_metas(vault_config_key: &Pubkey) -> Result<Vec<ExtraAccountMeta>> {
        Ok(vec![ExtraAccountMeta::new_with_pubkey(
            vault_config_key,
            false, // is_signer
            false, // is_writable
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

    /// CHECK: This account is initialized in the initialize_extra_account_metas instruction
    /// and its data is structured according to spl_tlv_account_resolution::state::ExtraAccountMetaList.
    /// The Token2022 program validates this account during CPI.
    #[account(
        seeds = [EXTRA_ACCOUNT_METAS_SEED, mint.key().as_ref()],
        bump
    )]
    pub extra_account_metas: AccountInfo<'info>,

    #[account(
        seeds = [VtokenConfig::SEED, mint.key().as_ref()],
        bump
    )]
    pub vtoken_config: Account<'info, VtokenConfig>,
}

#[derive(Accounts)]
pub struct SetVtokenIsTransferrableAccounts<'info> {
    #[account(mut)]
    pub manager: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [VtokenConfig::SEED, mint.key().as_ref()],
        bump
    )]
    pub vtoken_config: Account<'info, VtokenConfig>,
}
