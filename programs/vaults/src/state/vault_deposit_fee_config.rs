use anchor_lang::prelude::*;

#[account]
pub struct VaultDepositFeeConfig {
    pub vault: Pubkey,
    pub enabled: bool,
    pub fee_rate: u16, // Basis points (10000 = 100%), e.g., 50 = 0.5%
    pub platform_fee_recipient: Pubkey,
    pub bump: u8,
}

impl VaultDepositFeeConfig {
    pub const MAX_SPACE: usize = 8  // discriminator
        + 32 // vault pubkey
        + 1  // enabled (bool)
        + 2  // fee_rate (u16)
        + 32 // platform_fee_recipient pubkey
        + 1; // bump

    pub const SEED: &'static [u8] = b"VAULT_DEPOSIT_FEE_CONFIG";
    pub const MAX_FEE_RATE: u16 = 10000; // 100% max

    pub fn initialize(
        &mut self,
        vault: Pubkey,
        enabled: bool,
        fee_rate: u16,
        platform_fee_recipient: Pubkey,
        bump: u8,
    ) -> Result<()> {
        require!(
            fee_rate <= Self::MAX_FEE_RATE,
            VaultDepositFeeConfigError::InvalidFeeRate
        );

        self.vault = vault;
        self.enabled = enabled;
        self.fee_rate = fee_rate;
        self.platform_fee_recipient = platform_fee_recipient;
        self.bump = bump;

        Ok(())
    }

    pub fn update(
        &mut self,
        enabled: bool,
        fee_rate: u16,
        platform_fee_recipient: Pubkey,
    ) -> Result<()> {
        require!(
            fee_rate <= Self::MAX_FEE_RATE,
            VaultDepositFeeConfigError::InvalidFeeRate
        );

        self.enabled = enabled;
        self.fee_rate = fee_rate;
        self.platform_fee_recipient = platform_fee_recipient;

        Ok(())
    }

    pub fn enable(&mut self) -> Result<()> {
        self.enabled = true;
        Ok(())
    }

    pub fn disable(&mut self) -> Result<()> {
        self.enabled = false;
        Ok(())
    }

    /// Calculate the deposit fee amount based on the deposit amount
    /// Returns the fee amount in the same units as the deposit amount
    pub fn calculate_fee_amount(&self, deposit_amount: u64) -> Result<u64> {
        if !self.enabled || self.fee_rate == 0 {
            return Ok(0);
        }

        // Calculate fee: deposit_amount * fee_rate / 10000
        // Using u128 to prevent overflow
        (deposit_amount as u128)
            .checked_mul(self.fee_rate as u128)
            .ok_or(error!(VaultDepositFeeConfigError::NumericOverflow))?
            .checked_div(10000u128)
            .ok_or(error!(VaultDepositFeeConfigError::NumericOverflow))
            .map(|result| result as u64)
    }

    /// Check if the deposit fee config is initialized
    pub fn is_initialized(&self) -> bool {
        self.vault != Pubkey::default()
    }
}

#[error_code]
pub enum VaultDepositFeeConfigError {
    #[msg("Invalid fee rate. Must be between 0 and 10000 basis points (0-100%).")]
    InvalidFeeRate,
    #[msg("Vault mismatch in deposit fee config.")]
    VaultMismatch,
    #[msg("Account mismatch: platform fee recipient token account does not match fee config.")]
    PlatformFeeRecipientMismatch,
    #[msg("Numeric overflow in fee calculation.")]
    NumericOverflow,
}

