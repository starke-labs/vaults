use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, rent::Rent, system_instruction::transfer},
};
use anchor_spl::token_interface::{
    token_metadata_initialize, Mint, Token2022, TokenMetadataInitialize,
};

use crate::state::Vault;

pub fn initialize_token_metadata<'info>(
    vault_token_mint: InterfaceAccount<'info, Mint>,
    vault: Account<'info, Vault>,
    symbol: String,
    uri: String,
    signer_seeds: &[&[&[u8]]],
    token_program: Program<'info, Token2022>,
) -> Result<()> {
    let cpi_accounts = TokenMetadataInitialize {
        token_program_id: token_program.to_account_info(),
        mint: vault_token_mint.to_account_info(),
        metadata: vault_token_mint.to_account_info(), // metadata account is the mint, since data is stored in mint
        mint_authority: vault.to_account_info(),
        update_authority: vault.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer_seeds);
    token_metadata_initialize(cpi_ctx, vault.name.clone(), symbol, uri)?;

    Ok(())
}

// TODO: Create a new utils module and move this function there
pub fn update_account_lamports_to_minimum_balance<'info>(
    account: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
) -> Result<()> {
    let extra_lamports = Rent::get()?.minimum_balance(account.data_len()) - account.get_lamports();
    if extra_lamports > 0 {
        invoke(
            &transfer(payer.key, account.key, extra_lamports),
            &[payer, account, system_program],
        )?;
    }
    Ok(())
}
