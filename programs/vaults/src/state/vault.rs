use anchor_lang::prelude::*;

use super::{InvestorTier, InvestorType, TokenWhitelist};
use crate::{
    controllers::{
        compute_token_value_usd, parse_vault_balances, transform_price_to_aum_decimals,
        verify_price_update_and_get_pyth_price,
    },
    state::InvestorTypeWithRange,
};
use std::mem::size_of;

#[account]
pub struct Vault {
    pub manager: Pubkey,
    pub deposit_token_mint: Pubkey,
    pub name: String,
    pub bump: u8,
    pub mint: Pubkey, // vault token mint
    pub mint_bump: u8,

    // AUM configuration
    pub max_allowed_aum: u64, // In AUM_DECIMALS decimals, 0 means no maximum
    pub initial_vtoken_price: u32,

    // Investor permission settings
    pub allowed_investor_types: u16,
    pub allowed_investor_tiers: u16,
    pub range_allowed_per_investor_type: Vec<InvestorTypeWithRange>,

    // Deposit configuration
    pub max_depositors: u32, // 0 means unlimited
    pub current_depositors: u32,

    // State
    pub state: VaultState,

    // Fees
    pub last_fees_paid_timestamp: i64, // 0 means never. Resets to 0 when the vault is closed.
    pub management_fees_rate: u16,     // percentage, 2 decimals
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Default)]
#[repr(u8)]
pub enum VaultState {
    #[default]
    Active,
    DepositPaused,
}

impl Vault {
    pub const MAX_SPACE: usize = 8  // discriminator
        + size_of::<Pubkey>()       // manager pubkey
        + size_of::<Pubkey>()       // deposit token mint pubkey
        + 4                         // name length (u32)
        + 32                        // name (max 32 bytes)
        + 1                         // bump
        + size_of::<Pubkey>()       // vtoken mint pubkey
        + 1                         // vtoken mint bump
        + size_of::<u64>()          // max allowed AUM
        + size_of::<u32>()          // initial vtoken price
        + size_of::<u16>()          // allowed investor types
        + size_of::<u16>()          // allowed investor tiers
        + 4                         // range_allowed_per_investor_type vector length
        + InvestorTypeWithRange::MAX_SPACE * Self::MAX_RANGE_ENTRIES // range_allowed_per_investor_type vector
        + size_of::<u32>()          // max depositors
        + size_of::<u32>()          // current depositors
        + size_of::<VaultState>()   // vault state
        + size_of::<i64>()          // last fees paid timestamp
        + size_of::<u16>(); // management fees rate

    pub const MAX_RANGE_ENTRIES: usize = 16;
    pub const SEED: &'static [u8] = b"STARKE_VAULT";
    pub const VTOKEN_MINT_SEED: &'static [u8] = b"STARKE_VTOKEN_MINT";
    pub const MAX_MANAGEMENT_FEE_RATE: u16 = 10000;

    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        &mut self,
        manager: Pubkey,
        deposit_token_mint: Pubkey,
        name: String,
        bump: u8,
        vtoken_mint: Pubkey,
        vtoken_mint_bump: u8,
        max_allowed_aum: u64,
        initial_vtoken_price: u32,
        allowed_investor_types: u16,
        allowed_investor_tiers: u16,
        range_allowed_per_investor_type: Vec<InvestorTypeWithRange>,
        max_depositors: u32,
        management_fees_rate: u16,
    ) -> Result<()> {
        require!(initial_vtoken_price > 0, VaultError::InvalidInitialPrice);
        require!(name.len() <= 32, VaultError::NameTooLong);
        require!(!name.is_empty(), VaultError::NameTooShort);
        require!(
            management_fees_rate <= Self::MAX_MANAGEMENT_FEE_RATE,
            VaultError::InvalidFee
        );
        require!(
            range_allowed_per_investor_type.len() <= Self::MAX_RANGE_ENTRIES,
            VaultError::TooManyRangeEntries
        );
        for r in &range_allowed_per_investor_type {
            require!(
                r.max_deposit == 0 || r.min_deposit <= r.max_deposit,
                VaultError::InvalidDepositRange
            );
        }

        self.manager = manager;
        self.deposit_token_mint = deposit_token_mint;
        self.name = name;
        self.bump = bump;
        self.mint = vtoken_mint;
        self.mint_bump = vtoken_mint_bump;
        self.max_allowed_aum = max_allowed_aum;
        self.initial_vtoken_price = initial_vtoken_price;
        self.allowed_investor_types = allowed_investor_types;
        self.allowed_investor_tiers = allowed_investor_tiers;
        self.range_allowed_per_investor_type = range_allowed_per_investor_type;
        self.max_depositors = max_depositors;
        self.current_depositors = 0;
        self.state = VaultState::Active;
        self.last_fees_paid_timestamp = 0;
        self.management_fees_rate = management_fees_rate;

        Ok(())
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

    /// Assets under management and vault's deposit token price in USD
    pub fn get_aum_with_deposit<'info>(
        &self,
        remaining_accounts: &'info [AccountInfo<'info>],
        whitelist: &Account<'info, TokenWhitelist>,
        vault_key: &Pubkey,
    ) -> Result<(u64, Option<u64>)> {
        let vault_balances = parse_vault_balances(remaining_accounts, whitelist, vault_key)?;
        let mut deposit_price = None;
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
                let value = compute_token_value_usd(
                    b.token_balance,
                    b.token_decimals,
                    price_in_aum_decimals,
                );
                if b.token_mint == self.deposit_token_mint {
                    // Return value of 1 token, to maintain price consistency
                    deposit_price =
                        compute_token_value_usd(1, b.token_decimals, price_in_aum_decimals).ok();
                }
                value
            })
            .sum::<Result<u64>>();

        Ok((aum?, deposit_price))
    }

    /// Validates if the vault can accept more deposits based on max AUM limit
    pub fn validate_max_aum(&self, current_aum: u64, deposit_value: u64) -> Result<()> {
        // Check max AUM if it's set
        if self.max_allowed_aum > 0 {
            let new_aum = current_aum
                .checked_add(deposit_value)
                .ok_or(VaultError::NumericOverflow)?;

            require!(new_aum <= self.max_allowed_aum, VaultError::MaxAumExceeded);
        }

        Ok(())
    }

    /// Validates if the user's investor type and tier are allowed in this vault
    pub fn validate_investor_type(
        &self,
        investor_type: InvestorType,
        investor_tier: InvestorTier,
    ) -> Result<()> {
        require!(
            (self.allowed_investor_types == 0
                || self.allowed_investor_types & investor_type as u16 > 0)
                && (self.allowed_investor_tiers == 0
                    || self.allowed_investor_tiers & investor_tier as u16 > 0),
            VaultError::InvestorTypeNotAllowed
        );
        Ok(())
    }

    /// Validates deposit amount based on investor type
    pub fn validate_deposit_amount_by_type(
        &self,
        amount: u64,
        investor_type: InvestorType,
    ) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);

        for r in &self.range_allowed_per_investor_type {
            if r.investor_type == investor_type {
                if r.min_deposit > 0 {
                    require!(amount >= r.min_deposit, VaultError::DepositBelowMinimum);
                }
                if r.max_deposit > 0 {
                    require!(amount <= r.max_deposit, VaultError::DepositAboveMaximum);
                }
                return Ok(());
            }
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

    /// Return true if fees were NOT paid in the current quarter
    pub fn can_pay_management_fees(&self, now: i64) -> bool {
        const SECS_PER_DAY: i64 = 86_400;
        const DAYS_IN_QUARTERS: [i64; 4] = [90, 91, 92, 92];
        const DAYS_IN_LEAP_QUARTERS: [i64; 4] = [91, 91, 92, 92];

        let to_year_quarter = |ts: i64| -> (i64, i64) {
            let mut days = ts / SECS_PER_DAY;

            let mut year = 1970;
            loop {
                let year_days = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    366
                } else {
                    365
                };

                if days < year_days {
                    break;
                }
                days -= year_days;
                year += 1;
            }

            let mut quarter = 0;
            for m in if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                DAYS_IN_LEAP_QUARTERS
            } else {
                DAYS_IN_QUARTERS
            } {
                if days < m {
                    break;
                }
                days -= m;
                quarter += 1;
            }

            (year, quarter)
        };

        let (last_year, last_quarter) = to_year_quarter(self.last_fees_paid_timestamp);
        let (curr_year, curr_quarter) = to_year_quarter(now);

        last_year != curr_year || last_quarter != curr_quarter
    }

    pub fn pause_deposits(&mut self) {
        self.state = VaultState::DepositPaused;
    }
    pub fn resume_deposits(&mut self) {
        self.state = VaultState::Active;
    }
}

#[error_code]
pub enum VaultError {
    #[msg("Invalid deposit token")]
    InvalidDepositToken,
    #[msg("Name must be 1 character or more")]
    NameTooShort,
    #[msg("Name must be 32 characters or less")]
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
    #[msg("No outstanding vtoken supply to collect fees against")]
    NoVtokenSupply,
    #[msg("Invalid initial price, must be greater than 0")]
    InvalidInitialPrice,
    #[msg("User token account not found")]
    UserTokenAccountNotFound,
    #[msg("Unauthorized: this instruction can only be called by the authority.")]
    Unauthorized,
    #[msg("Fees not due yet.")]
    FeesNotDue,
    #[msg("Invalid vtoken mint")]
    InvalidVtokenMint,
    #[msg("Vault deposits are paused.")]
    DepositsPaused,
    #[msg("Deposit amount is above maximum")]
    DepositAboveMaximum,
    #[msg("Withdraw amount too small.")]
    WithdrawAmountTooSmall,
    #[msg("Deposit token supply is zero.")]
    DepositTokenSupplyZero,
    #[msg("Too many range entries, maximum is 16")]
    TooManyRangeEntries,
    #[msg("Invalid deposit range: min_deposit exceeds max_deposit")]
    InvalidDepositRange,
}

#[error_code]
pub enum VaultCloseError {
    #[msg("Cannot close vault: vtoken supply is not zero.")]
    NonZeroVtokenSupply,
    #[msg("Unauthorized: only the vault manager can close this vault.")]
    UnauthorizedManager,
}
