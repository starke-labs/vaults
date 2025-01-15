use anchor_lang::prelude::*;

use super::{TokenWhitelist, VaultFeesUpdated};
use crate::controllers::{
    compute_token_value_usd, get_token_price_from_pyth_feed, parse_vault_balances,
    transform_price_to_nav_decimals,
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

    pub const SEED: &'static [u8] = b"STARKE_VAULT";
    pub const VAULT_TOKEN_MINT_SEED: &'static [u8] = b"STARKE_VAULT_TOKEN_MINT";

    pub const FEE_UPDATE_DELAY: i64 = 30 * 24 * 60 * 60; // 30 days in seconds
    pub const MAX_FEE: u16 = 10000;

    pub fn initialize(
        &mut self,
        manager: Pubkey,
        deposit_token_mint: Pubkey,
        name: String,
        bump: u8,
        vault_token_mint: Pubkey,
        vault_token_mint_bump: u8,
        entry_fee: u16,
        exit_fee: u16,
    ) -> Result<()> {
        require!(name.len() <= 32, VaultError::NameTooLong);
        require!(name.len() > 0, VaultError::NameTooShort);
        require!(entry_fee <= Self::MAX_FEE, VaultError::InvalidFee);
        require!(exit_fee <= Self::MAX_FEE, VaultError::InvalidFee);

        self.manager = manager;
        self.deposit_token_mint = deposit_token_mint;
        self.name = name;
        self.bump = bump;
        self.mint = vault_token_mint;
        self.mint_bump = vault_token_mint_bump;
        self.entry_fee = entry_fee;
        self.exit_fee = exit_fee;
        self.pending_entry_fee = None;
        self.pending_exit_fee = None;
        self.fee_update_timestamp = 0;

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

    pub fn get_fees(&mut self, current_timestamp: i64, vault_key: Pubkey) -> Result<(u16, u16)> {
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
                    vault: vault_key,
                    manager: self.manager,
                    new_entry_fee: pending_entry,
                    new_exit_fee: pending_exit,
                    timestamp: current_timestamp,
                });
            }
        }

        Ok((self.entry_fee, self.exit_fee))
    }

    pub fn get_nav<'info>(
        &self,
        vault_token_accounts: &'info [AccountInfo<'info>],
        whitelist: Box<Account<'info, TokenWhitelist>>,
        vault_key: Pubkey,
    ) -> Result<u64> {
        // msg!("get_nav called");
        let vault_balances = parse_vault_balances(vault_token_accounts, whitelist, vault_key)?;
        let nav = vault_balances
            .iter()
            .map(|b| {
                msg!("Token balance: {}", b.token_balance);
                msg!("Token decimals: {}", b.token_decimals);
                let token_price = get_token_price_from_pyth_feed(
                    b.price_feed_id.clone(),
                    b.price_update.clone(),
                )?;
                // TODO: Throw error if confidence interval is above threshold
                //       https://docs.pyth.network/price-feeds/best-practices#confidence-intervals
                // msg!(
                //     "Token price: {} {} {}",
                //     token_price.price,
                //     token_price.conf,
                //     token_price.exponent
                // );
                let price_in_nav_decimals = transform_price_to_nav_decimals(token_price)?;
                // msg!("Price in NAV decimals: {}", price);
                compute_token_value_usd(b.token_balance, b.token_decimals, price_in_nav_decimals)
            })
            .sum::<Result<u64>>()?;

        Ok(nav)
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
}
