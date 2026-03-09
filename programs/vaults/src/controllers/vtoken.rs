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
    if vtoken_supply == 0 || total_aum == 0 {
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

/// Calculates the amount of vtokens to mint based on the management fees rate
pub fn calculate_management_fees_vtokens_to_mint(vtoken_supply: u64, rate: u16) -> Result<u64> {
    if rate == 10000 {
        return Ok(vtoken_supply);
    }

    // If rate is in percentage, then the formula is:
    // vtoken_supply * rate / (1 - rate).
    // Or when normally, when using basis points, then the formula is:
    // vtoken_supply * rate / (10000 - rate).
    (vtoken_supply as u128)
        .checked_mul(rate as u128)
        .ok_or(error!(VaultError::NumericOverflow))?
        .checked_div((10000u16 - rate) as u128)
        .ok_or(error!(VaultError::NumericOverflow))
        .map(|result| result as u64)
}

/// Calculates the current token price (NAV per vToken) in AUM_DECIMALS
/// Formula: Token Price = AUM / vToken Supply
pub fn compute_token_price(aum: u64, vtoken_supply: u64) -> Result<u64> {
    require!(vtoken_supply > 0, VaultError::NoVtokenSupply);

    // Token price = AUM / vToken Supply
    // Both are already in AUM_DECIMALS, so result is also in AUM_DECIMALS
    (aum as u128)
        .checked_mul(1u128) // No scaling needed since both are in same decimals
        .ok_or(error!(VaultError::NumericOverflow))?
        .checked_div(vtoken_supply as u128)
        .ok_or(error!(VaultError::NumericOverflow))
        .map(|result| result as u64)
}

/// Calculates the amount of vtokens to mint as performance fee
/// Formula: Max(CurrentPrice - LastPrice, 0) × TotalSupply × (FeeRate / CurrentPrice)
/// Where:
/// - CurrentPrice: current token price in AUM_DECIMALS
/// - LastPrice: high-water mark (last_perf_fee_token_price) in AUM_DECIMALS
/// - TotalSupply: current vToken supply
/// - FeeRate: performance fee rate in basis points (1000 = 10%)
pub fn calculate_performance_fee_vtokens_to_mint(
    current_token_price: u64,
    last_perf_fee_token_price: u64,
    vtoken_supply: u64,
    fee_rate: u16,
) -> Result<u64> {
    // Check if current price exceeds high-water mark
    if current_token_price <= last_perf_fee_token_price {
        // No profit above high-water mark, no fee
        return Ok(0);
    }

    // Calculate price delta (profit above high-water mark)
    let price_delta = current_token_price
        .checked_sub(last_perf_fee_token_price)
        .ok_or(error!(VaultError::NumericOverflow))?;

    // Formula: (Price Delta × vToken Supply × Fee Rate) / (10,000 × Current Price)
    // This gives us the vTokens to mint
    let vtokens_to_mint = (price_delta as u128)
        .checked_mul(vtoken_supply as u128)
        .ok_or(error!(VaultError::NumericOverflow))?
        .checked_mul(fee_rate as u128)
        .ok_or(error!(VaultError::NumericOverflow))?
        .checked_div(10_000u128)
        .ok_or(error!(VaultError::NumericOverflow))?
        .checked_div(current_token_price as u128)
        .ok_or(error!(VaultError::NumericOverflow))?;

    Ok(vtokens_to_mint as u64)
}
