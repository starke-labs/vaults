use anchor_lang::prelude::*;

use super::{InvestorType, TokenWhitelist, VaultFeesUpdated};
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
    // Maximum amount of aum allowed in vault, if None, there is no maximum
    // USD value in AUM_DECIMALS decimals
    pub max_allowed_aum: Option<u64>, // None means no maximum
    // New investor permission settings
    pub allow_retail: bool,
    pub allow_accredited: bool,
    pub allow_institutional: bool,
    pub allow_qualified: bool,
    // Deposit configuration (0 = no minimum)
    pub individual_min_deposit: u32, // For retail/accredited investors, 0 = no minimum
    pub institutional_min_deposit: u32, // For institutional/qualified investors, 0 = no minimum
    // Max depositors (0 = unlimited)
    pub max_depositors: u32, // 0 means unlimited
    pub current_depositors: u32,

    pub initial_vtoken_price: u32,
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
        + 8  // fee_update_timestamp (i64)
        + 9  // max_allowed_aum (Option<u64>)
        + 1  // allow_retail (bool)
        + 1  // allow_accredited (bool)
        + 1  // allow_institutional (bool)
        + 1  // allow_qualified (bool)
        + 4  // individual_min_deposit (u32)
        + 4  // institutional_min_deposit (u32)
        + 4  // max_depositors (u32)
        + 4; // current_depositors (u32)

    pub const SEED: &'static [u8] = b"STARKE_VAULT";
    pub const VTOKEN_MINT_SEED: &'static [u8] = b"STARKE_VTOKEN_MINT";

    pub const FEE_UPDATE_DELAY: i64 = 30 * 24 * 60 * 60; // 30 days in seconds
    pub const MAX_FEE: u16 = 10000;

    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        &mut self,
        manager: Pubkey,
        deposit_token_mint: Pubkey,
        name: String,
        bump: u8,
        vtoken_mint: Pubkey,
        vtoken_mint_bump: u8,
        max_allowed_aum: Option<u64>,
        allow_retail: bool,
        allow_accredited: bool,
        allow_institutional: bool,
        allow_qualified: bool,
        individual_min_deposit: u32,
        institutional_min_deposit: u32,
        max_depositors: u32,
        initial_vtoken_price: u32,
    ) -> Result<()> {
        require!(initial_vtoken_price > 0, VaultError::InvalidInitialPrice);
        require!(name.len() <= 32, VaultError::NameTooLong);
        require!(!name.is_empty(), VaultError::NameTooShort);

        self.manager = manager;
        self.deposit_token_mint = deposit_token_mint;
        self.name = name.to_string();
        self.bump = bump;
        self.mint = vtoken_mint;
        self.mint_bump = vtoken_mint_bump;
        self.entry_fee = 0;
        self.exit_fee = 0;
        self.pending_entry_fee = None;
        self.pending_exit_fee = None;
        self.fee_update_timestamp = 0;
        self.max_allowed_aum = max_allowed_aum;
        self.allow_retail = allow_retail;
        self.allow_accredited = allow_accredited;
        self.allow_institutional = allow_institutional;
        self.allow_qualified = allow_qualified;
        self.individual_min_deposit = individual_min_deposit;
        self.institutional_min_deposit = institutional_min_deposit;
        self.max_depositors = max_depositors;
        self.current_depositors = 0;
        self.initial_vtoken_price = initial_vtoken_price;

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


    /// Validates if the vault can accept more deposits based on max AUM limit
    pub fn validate_max_aum(&self, current_aum: u64, deposit_value: u64) -> Result<()> {
        // Check max AUM if it's set
        if let Some(max_aum) = self.max_allowed_aum {
            let new_aum = current_aum
                .checked_add(deposit_value)
                .ok_or(VaultError::NumericOverflow)?;

            require!(new_aum <= max_aum, VaultError::MaxAumExceeded);
        }

        Ok(())
    }

    /// Validates if the user's investor type is allowed in this vault
    pub fn validate_investor_type(&self, investor_type: &InvestorType) -> Result<()> {
        let is_allowed = match investor_type {
            InvestorType::Retail => self.allow_retail,
            InvestorType::Accredited => self.allow_accredited,
            InvestorType::Institutional => self.allow_institutional,
            InvestorType::Qualified => self.allow_qualified,
        };

        require!(is_allowed, VaultError::InvestorTypeNotAllowed);
        Ok(())
    }

    /// Validates deposit amount based on investor type
    pub fn validate_deposit_amount_by_type(
        &self,
        amount: u64,
        investor_type: &InvestorType,
    ) -> Result<()> {
        // Check if amount is not zero
        require!(amount > 0, VaultError::InvalidAmount);

        // Check minimum deposit amount based on investor type (0 = no minimum)
        let min_deposit = match investor_type {
            InvestorType::Retail | InvestorType::Accredited => self.individual_min_deposit,
            InvestorType::Institutional | InvestorType::Qualified => self.institutional_min_deposit,
        };

        if min_deposit > 0 {
            require!(amount >= min_deposit as u64, VaultError::DepositBelowMinimum);
        }

        Ok(())
    }

    /// Validates if the vault can accept more depositors
    pub fn validate_max_depositors(&self, is_new_depositor: bool) -> Result<()> {
        if is_new_depositor && self.max_depositors > 0 {
            require!(
                self.current_depositors < self.max_depositors,
                VaultError::MaxDepositorsExceeded
            );
        }
        Ok(())
    }

    /// Increments the depositor count (call when a new depositor makes their first deposit)
    pub fn increment_depositor_count(&mut self) -> Result<()> {
        self.current_depositors = self
            .current_depositors
            .checked_add(1)
            .ok_or(VaultError::NumericOverflow)?;
        Ok(())
    }

    /// Decrements the depositor count (call when a depositor withdraws all their tokens)
    pub fn decrement_depositor_count(&mut self) -> Result<()> {
        self.current_depositors = self
            .current_depositors
            .checked_sub(1)
            .ok_or(VaultError::NumericOverflow)?;
        Ok(())
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
    #[msg("Maximum AUM limit exceeded")]
    MaxAumExceeded,
    #[msg("Investor type not allowed for this vault")]
    InvestorTypeNotAllowed,
    #[msg("Maximum depositors limit exceeded")]
    MaxDepositorsExceeded,
    #[msg("Cannot close vault with active depositors")]
    VaultHasActiveDepositors,
    #[msg("Cannot close vault with outstanding vtokens")]
    VTokensOutstanding,
    #[msg("Cannot close vault with remaining funds")]
    FundsRemaining,
    #[msg("Invalid initial price, must be greater than 0")]
    InvalidInitialPrice,
}


#[error_code]
pub enum VaultCloseError {
    #[msg("Cannot close vault: vtoken supply is not zero.")]
    NonZeroVtokenSupply,
    #[msg("Unauthorized: only the vault manager can close this vault.")]
    UnauthorizedManager,
}
