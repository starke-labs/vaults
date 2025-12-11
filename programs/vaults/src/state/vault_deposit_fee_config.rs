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
}

#[error_code]
pub enum VaultDepositFeeConfigError {
    #[msg("Invalid fee rate. Must be between 0 and 10000 basis points (0-100%).")]
    InvalidFeeRate,
    #[msg("Vault mismatch in deposit fee config.")]
    VaultMismatch,
}

