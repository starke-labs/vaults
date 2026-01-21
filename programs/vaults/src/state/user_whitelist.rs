use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum InvestorType {
    Retail,
    Accredited,
    Institutional,
    Qualified,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UserEntry {
    pub user: Pubkey,
    pub investor_type: InvestorType,
}

#[account]
pub struct UserWhitelist {
    pub authority: Pubkey,
    pub bump: u8,
    pub users: Vec<UserEntry>,
}

impl UserWhitelist {
    pub const MAX_USERS: usize = 300;
    pub const MAX_SPACE: usize = 8 // discriminator
        + 32 // authority
        + 1  // bump
        + 4  // users vector length
        + (UserEntry::MAX_SPACE * Self::MAX_USERS); // users vector

    pub const SEED: &'static [u8] = b"STARKE_USER_WHITELIST";

    pub fn initialize(&mut self, authority: Pubkey, bump: u8) -> Result<()> {
        self.authority = authority;
        self.bump = bump;
        self.users = Vec::new();
        Ok(())
    }

    pub fn add_user(&mut self, user: Pubkey, investor_type: InvestorType) -> Result<()> {
        require!(
            self.users.len() < Self::MAX_USERS,
            UserWhitelistError::WhitelistFull
        );

        // Check if user already exists
        if let Some(existing_user) = self.users.iter_mut().find(|u| u.user == user) {
            // Update existing user's investor type
            existing_user.investor_type = investor_type;
        } else {
            // Add new user
            self.users.resize(
                self.users.len() + 1,
                UserEntry {
                    user,
                    investor_type,
                },
            );
        }

        Ok(())
    }

    pub fn remove_user(&mut self, user: Pubkey) -> Result<()> {
        let initial_len = self.users.len();
        self.users.retain(|u| u.user != user);

        require!(
            self.users.len() < initial_len,
            UserWhitelistError::UserNotFound
        );

        Ok(())
    }

    pub fn get_user_type(&self, user: &Pubkey) -> Option<InvestorType> {
        self.users
            .iter()
            .find(|u| u.user == *user)
            .map(|u| u.investor_type.clone())
    }

    pub fn is_user_whitelisted(&self, user: &Pubkey) -> bool {
        self.users.iter().any(|u| u.user == *user)
    }
}

impl UserEntry {
    pub const MAX_SPACE: usize = 32 // user pubkey
        + 1; // investor_type enum (u8)
}

#[error_code]
pub enum UserWhitelistError {
    #[msg("User whitelist is full")]
    WhitelistFull,
    #[msg("User not found in whitelist")]
    UserNotFound,
    #[msg("User not whitelisted")]
    UserNotWhitelisted,
    #[msg("Invalid investor type for this vault")]
    InvalidInvestorType,
    #[msg("Unauthorized access")]
    UnauthorizedAccess,
}
