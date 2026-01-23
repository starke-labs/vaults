use anchor_lang::prelude::*;

#[account]
pub struct UserDepositInfo {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub first_deposit_timestamp: i64, // When user first deposited
    pub lock_in_period_seconds: u64,  // Lock-in period that applied at time of first deposit (0 = no lock-in)
    pub bump: u8,
}

impl UserDepositInfo {
    pub const MAX_SPACE: usize = 8  // discriminator
        + 32 // user pubkey
        + 32 // vault pubkey
        + 8  // first_deposit_timestamp (i64)
        + 8  // lock_in_period_seconds (u64)
        + 1; // bump (u8)

    pub const SEED: &'static [u8] = b"USER_DEPOSIT_INFO";

    pub fn initialize(
        &mut self,
        user: Pubkey,
        vault: Pubkey,
        first_deposit_timestamp: i64,
        lock_in_period_seconds: u64,
        bump: u8,
    ) {
        self.user = user;
        self.vault = vault;
        self.first_deposit_timestamp = first_deposit_timestamp;
        self.lock_in_period_seconds = lock_in_period_seconds;
        self.bump = bump;
    }

    /// Check if the lock-in period has expired
    pub fn is_lock_in_expired(&self, current_timestamp: i64) -> bool {
        if self.lock_in_period_seconds == 0 {
            return true; // No lock-in period
        }

        let elapsed = current_timestamp
            .checked_sub(self.first_deposit_timestamp)
            .unwrap_or(0);

        elapsed >= self.lock_in_period_seconds as i64
    }
}

