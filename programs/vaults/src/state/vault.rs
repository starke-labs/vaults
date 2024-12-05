use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    pub manager: Pubkey,
    pub deposit_token_mint: Pubkey,
    pub name: String,
    pub bump: u8,
    pub mint: Pubkey, // vault token mint
    pub mint_bump: u8,
}

impl Vault {
    pub const MAX_SPACE: usize = 8  // discriminator
        + 32 // manager pubkey
        + 32 // deposit token mint pubkey
        + 4  // name length (u32)
        + 32 // name (max 32 bytes)
        + 1 // bump
        + 32 // vault token mint pubkey
        + 1; // vault token mint bump

    pub const SEED: &'static [u8] = b"STARKE_VAULT";
    pub const VAULT_TOKEN_MINT_SEED: &'static [u8] = b"STARKE_VAULT_TOKEN_MINT";

    pub fn initialize(
        &mut self,
        manager: Pubkey,
        deposit_token_mint: Pubkey,
        name: String,
        bump: u8,
        vault_token_mint: Pubkey,
        vault_token_mint_bump: u8,
    ) -> Result<()> {
        require!(name.len() <= 32, VaultError::NameTooLong);

        self.manager = manager;
        self.deposit_token_mint = deposit_token_mint;
        self.name = name;
        self.bump = bump;
        self.mint = vault_token_mint;
        self.mint_bump = vault_token_mint_bump;
        Ok(())
    }
}

#[error_code]
pub enum VaultError {
    #[msg("Invalid deposit token")]
    InvalidDepositToken,
    #[msg("Name must be 32 characters or less")]
    NameTooLong,
    #[msg("Numeric overflow")]
    NumericOverflow,
    #[msg("Insufficient funds for withdrawal")]
    InsufficientFunds,
}
