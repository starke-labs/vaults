use anchor_lang::prelude::*;

#[event]
pub struct VaultCreated {
    pub vault: Pubkey,
    pub manager: Pubkey,
    pub deposit_token: Pubkey,
    pub name: String,
    pub timestamp: i64,
}

#[event]
pub struct VaultUpdated {
    pub vault: Pubkey,
    pub new_name: Option<String>,
    pub new_deposit_token: Option<Pubkey>,
    pub timestamp: i64,
}

#[event]
pub struct DepositMade {
    pub vault: Pubkey,
    pub depositor: Pubkey,
    pub amount: u64,
    pub total_deposited: u64,
    pub timestamp: i64,
}
