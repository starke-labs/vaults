use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    pub manager: Pubkey,
    pub deposit_token: Pubkey,
    pub name: String,
    pub bump: u8,
}

impl Vault {
    pub const MAX_SPACE: usize = 8  // discriminator
        + 32 // manager pubkey
        + 32 // deposit token pubkey
        + 4  // name length
        + 32 // name
        + 1; // bump

    pub const SEED: &'static [u8] = b"STARKE_VAULT";

    pub fn initialize(
        &mut self,
        manager: Pubkey,
        deposit_token: Pubkey,
        name: String,
        bump: u8,
    ) -> Result<()> {
        require!(name.len() <= 32, VaultError::NameTooLong);

        self.manager = manager;
        self.deposit_token = deposit_token;
        self.name = name;
        self.bump = bump;

        Ok(())
    }
}

#[error_code]
pub enum VaultError {
    #[msg("Invalid deposit token")]
    InvalidDepositToken,
    #[msg("Name must be 32 characters or less")]
    NameTooLong,
}
