use anchor_lang::prelude::*;

#[event]
pub struct VaultCreated {
    pub vault: Pubkey,
    pub manager: Pubkey,
    pub deposit_token: Pubkey,
    pub vault_token_mint: Pubkey,
    pub name: String,
    pub timestamp: i64,
    pub entry_fee: u16,
    pub exit_fee: u16,
}

#[event]
pub struct VaultFeesUpdateRequested {
    pub vault: Pubkey,
    pub manager: Pubkey,
    pub pending_entry_fee: u16,
    pub pending_exit_fee: u16,
    pub timestamp: i64,
}

#[event]
pub struct VaultFeesUpdated {
    pub vault: Pubkey,
    pub manager: Pubkey,
    pub new_entry_fee: u16,
    pub new_exit_fee: u16,
    pub timestamp: i64,
}

#[event]
pub struct DepositMade {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawMade {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokenWhitelisted {
    pub token: Pubkey,
    pub price_feed_id: String,
    pub price_update: Pubkey,
    pub timestamp: i64,
}
