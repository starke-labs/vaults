use anchor_lang::prelude::*;

use crate::state::InvestorTypeWithRange;

#[event]
pub struct VaultCreated {
    pub vault: Pubkey,
    pub manager: Pubkey,
    pub deposit_token_mint: Pubkey,
    pub vtoken_mint: Pubkey,
    pub name: String,
    pub timestamp: i64,
    pub max_allowed_aum: u64, // 0 means no limit
    pub initial_vtoken_price: u32,
    pub allowed_investor_types: u16,
    pub allowed_investor_tiers: u16,
    pub range_allowed_per_investor_type: Vec<InvestorTypeWithRange>,
    pub max_depositors: u32,
    pub management_fee_rate: u16,
}

#[event]
pub struct Deposited {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub deposit_amount: u64,
    pub vtoken_mint: Pubkey,
    pub vtoken_minted_amount: u64,
    pub new_vtoken_supply: u64,
    pub timestamp: i64,
}

#[event]
pub struct Withdrawn {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub vtoken_mint: Pubkey,
    pub vtoken_burned_amount: u64,
    pub new_vtoken_supply: u64,
    pub timestamp: i64,
}

#[event]
pub struct ManagementFeeMinted {
    pub vault: Pubkey,
    pub manager: Pubkey,
    pub vtoken_mint: Pubkey,
    pub vtoken_fee_amount: u64,
    pub new_vtoken_supply: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawnInDepositToken {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub vtoken_mint: Pubkey,
    pub vtoken_burned_amount: u64,
    pub new_vtoken_supply: u64,
    pub timestamp: i64,
    pub deposit_token_mint: Pubkey,
}

#[event]
pub struct WhitelistTokenAdded {
    pub mint: Pubkey,
    pub price_feed_id: String,
    pub price_update: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct WhitelistTokenRemoved {
    pub mint: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct WhitelistManagerAdded {
    pub manager: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct WhitelistManagerRemoved {
    pub manager: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct VaultClosed {
    pub vault: Pubkey,
    pub manager: Pubkey,
    pub timestamp: i64,
}
