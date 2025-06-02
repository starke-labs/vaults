use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{
        mpl_token_metadata::{
            instructions::CreateV1CpiBuilder,
            types::{PrintSupply, TokenStandard},
        },
        Metadata,
    },
    token_2022::Token2022,
    token_interface::Mint,
};

use crate::constants::AUM_DECIMALS;

pub fn initialize_token_metadata<'info>(
    name: String,
    symbol: String,
    uri: String,
    payer: &Signer<'info>,
    metadata: &AccountInfo<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    mint_authority: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    instructions_sysvar: &AccountInfo<'info>,
    token_2022_program: &Program<'info, Token2022>,
    metadata_program: &Program<'info, Metadata>,
    system_program: &Program<'info, System>,
) -> Result<()> {
    CreateV1CpiBuilder::new(metadata_program)
        .metadata(metadata)
        .mint(&mint.to_account_info(), false)
        .authority(mint_authority)
        .payer(payer)
        .update_authority(mint_authority, true)
        .system_program(system_program)
        .sysvar_instructions(instructions_sysvar)
        .spl_token_program(Some(&token_2022_program.to_account_info()))
        .token_standard(TokenStandard::Fungible)
        .seller_fee_basis_points(0)
        .print_supply(PrintSupply::Zero)
        .name(name)
        .symbol(symbol)
        .uri(uri)
        .decimals(AUM_DECIMALS)
        .invoke_signed(signer_seeds)?;

    Ok(())
}
