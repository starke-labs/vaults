use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    pub manager: Pubkey,
    pub deposit_token: Pubkey,
    pub name: String,
    bump: u8,
}

impl Vault {}
