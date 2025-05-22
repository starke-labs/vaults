use anchor_lang::prelude::*;

use super::{TokenWhitelist, VaultFeesUpdated};
use crate::controllers::{
    compute_token_value_usd, parse_vault_balances, transform_price_to_aum_decimals,
    verify_price_update_and_get_pyth_price,
};

#[account]
pub struct Vault {
    pub manager: Pubkey,
    pub deposit_token_mint: Pubkey,
    pub name: String,
    pub bump: u8,
    pub mint: Pubkey, // vault token mint
    pub mint_bump: u8,
    // Fees
    pub entry_fee: u16, // percentage, 2 decimals, lowest is 1 (0.01%) and highest is 10000 (100%)
    pub exit_fee: u16,  // percentage, 2 decimals, lowest is 1 (0.01%) and highest is 10000 (100%)
    pub pending_entry_fee: Option<u16>,
    pub pending_exit_fee: Option<u16>,
    pub fee_update_timestamp: i64,
    // Config
    // if the vault is private, then we have the following configurability
    // pub is_private_vault: bool,
    // minimum amount of deposit token to deposit in deposit token decimals, if None, there is no minimum
    // pub min_deposit_amount: Option<u64>,
    // maximum amount of aum allowed in vault, if None, there is no maximum
    // pub max_allowed_aum: Option<u64>,
}

impl Vault {
    pub const MAX_SPACE: usize = 8  // discriminator
        + 32 // manager pubkey
        + 32 // deposit token mint pubkey
        + 4  // name length (u32)
        + 32 // name (max 32 bytes)
        + 1  // bump
        + 32 // vault token mint pubkey
        + 1  // vault token mint bump
        + 2  // entry fee (u16)
        + 2  // exit fee (u16)
        + 3  // pending_entry_fee (Option<u16>)
        + 3  // pending_exit_fee (Option<u16>)
        + 8; // fee_update_timestamp (i64)
             // + 9  // min_deposit_amount (Option<u64>)
             // + 9; // max_allowed_aum (Option<u64>)

    pub const SEED: &'static [u8] = b"STARKE_VAULT";
    pub const VTOKEN_MINT_SEED: &'static [u8] = b"STARKE_VTOKEN_MINT";

    pub const FEE_UPDATE_DELAY: i64 = 30 * 24 * 60 * 60; // 30 days in seconds
    pub const MAX_FEE: u16 = 10000;

    pub fn initialize(
        &mut self,
        manager: &Pubkey,
        deposit_token_mint: &Pubkey,
        name: &str,
        bump: u8,
        vtoken_mint: &Pubkey,
        vtoken_mint_bump: u8,
        entry_fee: u16,
        exit_fee: u16,
        // min_deposit_amount: u64,
    ) -> Result<()> {
        require!(name.len() <= 32, VaultError::NameTooLong);
        require!(name.len() > 0, VaultError::NameTooShort);
        require!(entry_fee <= Self::MAX_FEE, VaultError::InvalidFee);
        require!(exit_fee <= Self::MAX_FEE, VaultError::InvalidFee);

        self.manager = *manager;
        self.deposit_token_mint = *deposit_token_mint;
        self.name = name.to_string();
        self.bump = bump;
        self.mint = *vtoken_mint;
        self.mint_bump = vtoken_mint_bump;
        self.entry_fee = entry_fee;
        self.exit_fee = exit_fee;
        self.pending_entry_fee = None;
        self.pending_exit_fee = None;
        // self.fee_update_timestamp = 0;
        // self.min_deposit_amount = min_deposit_amount;

        Ok(())
    }

    pub fn update_fees(
        &mut self,
        new_entry_fee: u16,
        new_exit_fee: u16,
        current_timestamp: i64,
    ) -> Result<()> {
        require!(new_entry_fee <= Self::MAX_FEE, VaultError::InvalidFee);
        require!(new_exit_fee <= Self::MAX_FEE, VaultError::InvalidFee);

        // Only check delay if there was a previous fee update
        if self.fee_update_timestamp != 0 {
            require!(
                current_timestamp >= self.fee_update_timestamp + Self::FEE_UPDATE_DELAY,
                VaultError::FeeUpdateDelayNotPassed
            );
        }

        self.pending_entry_fee = Some(new_entry_fee);
        self.pending_exit_fee = Some(new_exit_fee);
        self.fee_update_timestamp = current_timestamp;

        Ok(())
    }

    pub fn get_fees(&mut self, current_timestamp: i64, vault_key: &Pubkey) -> Result<(u16, u16)> {
        if let (Some(pending_entry), Some(pending_exit)) =
            (self.pending_entry_fee, self.pending_exit_fee)
        {
            if current_timestamp >= self.fee_update_timestamp + Self::FEE_UPDATE_DELAY {
                // Update the fees if the delay period has passed
                self.entry_fee = pending_entry;
                self.exit_fee = pending_exit;
                self.pending_entry_fee = None;
                self.pending_exit_fee = None;
                self.fee_update_timestamp = 0;

                emit!(VaultFeesUpdated {
                    vault: *vault_key,
                    manager: self.manager,
                    new_entry_fee: pending_entry,
                    new_exit_fee: pending_exit,
                    timestamp: current_timestamp,
                });
            }
        }

        Ok((self.entry_fee, self.exit_fee))
    }

    /// Assets under management
    pub fn get_aum<'info>(
        &self,
        remaining_accounts: &'info [AccountInfo<'info>],
        whitelist: &Account<'info, TokenWhitelist>,
        vault_key: &Pubkey,
    ) -> Result<u64> {
        let vault_balances = parse_vault_balances(remaining_accounts, whitelist, vault_key)?;
        let aum = vault_balances
            .iter()
            .map(|b| {
                let token_price = verify_price_update_and_get_pyth_price(
                    whitelist,
                    &b.token_mint,
                    &b.price_update,
                )?;
                // TODO: Throw error if confidence interval is above threshold
                //       https://docs.pyth.network/price-feeds/best-practices#confidence-intervals
                let price_in_aum_decimals = transform_price_to_aum_decimals(&token_price)?;
                compute_token_value_usd(b.token_balance, b.token_decimals, price_in_aum_decimals)
            })
            .sum::<Result<u64>>()?;

        Ok(aum)
    }
}

#[error_code]
pub enum VaultError {
    #[msg("Invalid deposit token")]
    InvalidDepositToken,
    #[msg("Name must be 32 characters or less")]
    NameTooShort,
    #[msg("Name must be 1 character or more")]
    NameTooLong,
    #[msg("Numeric overflow")]
    NumericOverflow,
    #[msg("Insufficient funds for withdrawal")]
    InsufficientFunds,
    #[msg("Invalid fee")]
    InvalidFee,
    #[msg("Fee update delay not passed")]
    FeeUpdateDelayNotPassed,
    #[msg("Mint and token account mismatch")]
    MintAndTokenAccountMismatch,
    #[msg("Vault and token account mismatch")]
    VaultAndTokenAccountMismatch,
    #[msg("User and token account mismatch")]
    UserAndTokenAccountMismatch,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Price confidence too low")]
    PriceConfidenceTooLow,
    #[msg("Deposit amount is below minimum")]
    DepositBelowMinimum,
}
