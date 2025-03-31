use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata,
    },
    token_interface::Mint,
};

pub fn initialize_token_metadata<'info>(
    name: &str,
    symbol: &str,
    uri: &str,
    payer: &Signer<'info>,
    metadata: &AccountInfo<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    mint_authority: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    rent: &Sysvar<'info, Rent>,
    metadata_program: &Program<'info, Metadata>,
    system_program: &Program<'info, System>,
) -> Result<()> {
    let token_data: DataV2 = DataV2 {
        name: name.to_string(),
        symbol: symbol.to_string(),
        uri: uri.to_string(),
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    let metadata_ctx = CpiContext::new_with_signer(
        metadata_program.to_account_info(),
        CreateMetadataAccountsV3 {
            payer: payer.to_account_info(),
            update_authority: mint_authority.to_account_info(),
            mint: mint.to_account_info(),
            metadata: metadata.to_account_info(),
            mint_authority: mint_authority.to_account_info(),
            system_program: system_program.to_account_info(),
            rent: rent.to_account_info(),
        },
        signer_seeds,
    );

    create_metadata_accounts_v3(metadata_ctx, token_data, false, true, None)?;

    Ok(())
}
