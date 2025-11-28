use anchor_lang::prelude::*;

#[event]
pub struct VaultCreated {
    pub vault: Pubkey,
    pub manager: Pubkey,
    pub deposit_token_mint: Pubkey,
    pub vtoken_mint: Pubkey,
    pub name: String,
    pub timestamp: i64,
    pub max_allowed_aum: Option<u64>,
    pub initial_vtoken_price: u32,
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
pub struct ManagementFeeCollected {
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub recipient: Pubkey,
    pub vtoken_mint: Pubkey,
    pub vtoken_fee_amount: u64,
    pub new_vtoken_supply: u64,
    pub timestamp: i64,
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
