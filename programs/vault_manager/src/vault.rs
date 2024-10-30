use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    pub manager: Pubkey,
    pub deposit_token: Pubkey,
    pub name: String,
    pub bump: u8,
}

// TODO: add error handling
impl Vault {
    pub fn update(&mut self, name: Option<String>, deposit_token: Option<Pubkey>) -> Result<()> {
        if let Some(name) = name {
            self.name = name;
            msg!("Vault name changed to: {}", self.name)
        }

        // TODO: this should not be allowed
        if let Some(deposit_token) = deposit_token {
            self.deposit_token = deposit_token;
            msg!("Vault deposit token changed to: {}", self.deposit_token);
        }
        Ok(())
    }
}
