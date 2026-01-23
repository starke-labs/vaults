use anchor_lang::prelude::*;

#[account]
pub struct WithdrawalRequest {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub vtoken_mint: Pubkey,
    pub amount: u64,
    pub requested_at: i64,
    pub bump: u8,
}

impl WithdrawalRequest {
    pub const SEED: &'static [u8] = b"withdrawal_request";

    pub const MAX_SPACE: usize = 8   // discriminator
        + 32  // user
        + 32  // vault
        + 32  // vtoken_mint
        + 8   // amount
        + 8   // requested_at (i64)
        + 1;  // bump
}
