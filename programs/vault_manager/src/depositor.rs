use anchor_lang::prelude::*;

#[account]
pub struct Depositor {
    pub vault: Pubkey,
    pub depositor: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

impl Depositor {
    pub const MAX_SPACE: usize = 8  // discriminator
        + 32 // vault pubkey
        + 32 // depositor pubkey
        + 8  // amount (u64)
        + 1; // bump

    pub fn initialize(&mut self, vault: Pubkey, depositor: Pubkey, bump: u8) {
        self.vault = vault;
        self.depositor = depositor;
        self.amount = 0;
        self.bump = bump;
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        self.amount = self
            .amount
            .checked_add(amount)
            .ok_or(DepositorError::NumericOverflow)?;

        Ok(())
    }
}

#[error_code]
pub enum DepositorError {
    #[msg("Numeric overflow occurred")]
    NumericOverflow,
}
