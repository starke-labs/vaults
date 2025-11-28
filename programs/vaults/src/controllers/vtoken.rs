use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{burn, mint_to, Burn, Mint, MintTo, TokenAccount},
};

use crate::state::{Vault, VaultError};

pub fn mint_vtoken<'info>(
    vault: &Account<'info, Vault>,
    mint: &InterfaceAccount<'info, Mint>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
    token_program: &Program<'info, Token2022>,
) -> Result<()> {
    let mint_accounts = MintTo {
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: vault.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(token_program.to_account_info(), mint_accounts, signer_seeds);
    mint_to(cpi_ctx, amount)?;

    Ok(())
}

pub fn burn_vtoken<'info>(
    user: &Signer<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    from: &InterfaceAccount<'info, TokenAccount>,
    amount: u64,
    signer_seeds: &[&[&[u8]]],
    token_program: &Program<'info, Token2022>,
) -> Result<()> {
    let burn_accounts = Burn {
        mint: mint.to_account_info(),
        from: from.to_account_info(),
        authority: user.to_account_info(),
    };
    let cpi_ctx =
        CpiContext::new_with_signer(token_program.to_account_info(), burn_accounts, signer_seeds);
    burn(cpi_ctx, amount)?;

    Ok(())
}

pub fn calculate_vtokens_to_mint(
    total_aum: u64,
    deposit_value: u64,
    vtoken_supply: u64,
    initial_vtoken_price: u32,
) -> Result<u64> {
    if vtoken_supply == 0 {
        // Initial deposit

        // NOTE: Added initial_vtoken_price
        // The program was upgraded to add intial_vtoken_price.
        // Therefore, older vaults will have initial_vtoken_price = 0.
        // In this case, we want to mint the same amount of vtokens as the deposit amount.
        // This is the same as having initial_vtoken_price = 1.
        Ok(deposit_value
            .checked_div(initial_vtoken_price as u64)
            .unwrap_or(deposit_value))
    } else {
        // Calculate proportional amount based on AUM
        (deposit_value as u128)
            .checked_mul(vtoken_supply as u128)
            .ok_or(error!(VaultError::NumericOverflow))?
            .checked_div(total_aum as u128)
            .ok_or(error!(VaultError::NumericOverflow))
            .map(|result| result as u64)
    }
}

/// Calculates 0.25% (25 basis points) of vToken supply for management fee
pub fn calculate_management_fee_vtokens(vtoken_supply: u64) -> Result<u64> {
    if vtoken_supply == 0 {
        return Ok(0);
    }

    // 0.25% = 25 basis points
    const FEE_BPS: u128 = 25;
    const BASIS_POINTS_DENOMINATOR: u128 = 10_000;

    let denominator = BASIS_POINTS_DENOMINATOR
        .checked_sub(FEE_BPS)
        .ok_or(error!(VaultError::NumericOverflow))?;

    let mut amount = (vtoken_supply as u128)
        .checked_mul(FEE_BPS)
        .ok_or(error!(VaultError::NumericOverflow))?
        .checked_div(denominator)
        .ok_or(error!(VaultError::NumericOverflow))?;

    if amount == 0 {
        amount = 1;
    }

    Ok(amount as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn management_fee_rounds_up() {
        let supply = 1_000_000u64;
        let fee = calculate_management_fee_vtokens(supply).unwrap();
        assert_eq!(fee, 2507);
    }

    #[test]
    fn management_fee_zero_supply() {
        assert_eq!(calculate_management_fee_vtokens(0).unwrap(), 0);
    }
}
