use anchor_lang::prelude::*;

#[account]
pub struct VaultBalance {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

impl VaultBalance {
    pub const MAX_SPACE: usize = 8  // discriminator
        + 32 // vault pubkey
        + 32 // user pubkey
        + 8  // amount (u64)
        + 1; // bump

    pub fn initialize(&mut self, vault: Pubkey, user: Pubkey, bump: u8) {
        self.vault = vault;
        self.user = user;
        self.amount = 0;
        self.bump = bump;
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        self.amount = self
            .amount
            .checked_add(amount)
            .ok_or(VaultBalanceError::NumericOverflow)?;

        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(self.amount >= amount, VaultBalanceError::InsufficientFunds);

        self.amount = self
            .amount
            .checked_sub(amount)
            .ok_or(VaultBalanceError::NumericOverflow)?;

        Ok(())
    }
}

#[error_code]
pub enum VaultBalanceError {
    #[msg("Numeric overflow occurred")]
    NumericOverflow,
    #[msg("Insufficient funds for withdrawal")]
    InsufficientFunds,
}
