use anchor_lang::prelude::*;

#[account]
pub struct ManagerWhitelist {
    pub managers: Vec<Pubkey>,
    pub bump: u8,
}

impl ManagerWhitelist {
    // NOTE: Adjust MAX_MANAGERS as needed, considering account size limits
    pub const MAX_MANAGERS: usize = 100;
    pub const MAX_SPACE: usize = 8 + // discriminator
        4 + // vec length
        (32 * Self::MAX_MANAGERS) + // managers (100 max)
        1; // bump

    pub const SEED: &'static [u8] = b"STARKE_MANAGER_WHITELIST";

    pub fn initialize(&mut self, bump: u8) -> Result<()> {
        self.managers = Vec::new();
        self.bump = bump;
        Ok(())
    }

    pub fn add_manager(&mut self, manager_pubkey: &Pubkey) -> Result<()> {
        require!(
            !self.managers.iter().any(|m| m == manager_pubkey),
            ManagerWhitelistError::ManagerAlreadyWhitelisted
        );
        require!(
            self.managers.len() < Self::MAX_MANAGERS,
            ManagerWhitelistError::WhitelistFull
        );

        self.managers.push(*manager_pubkey);
        Ok(())
    }

    pub fn remove_manager(&mut self, manager_pubkey: &Pubkey) -> Result<()> {
        if let Some(index) = self.managers.iter().position(|m| m == manager_pubkey) {
            self.managers.remove(index);
            Ok(())
        } else {
            err!(ManagerWhitelistError::ManagerNotWhitelisted)
        }
    }

    pub fn is_whitelisted(&self, manager_pubkey: &Pubkey) -> bool {
        self.managers.iter().any(|m| m == manager_pubkey)
    }
}

#[error_code]
pub enum ManagerWhitelistError {
    #[msg("Manager is already whitelisted")]
    ManagerAlreadyWhitelisted,
    #[msg("Manager is not whitelisted")]
    ManagerNotWhitelisted,
    #[msg("Manager whitelist is full")]
    WhitelistFull,
    #[msg("Only starke authority can perform this action")]
    UnauthorizedAccess,
    #[msg("Manager Whitelist already initialized")]
    WhitelistAlreadyInitialized,
}
