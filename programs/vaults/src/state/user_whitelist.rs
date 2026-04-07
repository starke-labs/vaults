use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
pub enum InvestorType {
    Unknown = 1 << 0,
    Entity = 1 << 1,
    Individual = 1 << 2,
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug, PartialEq)]
pub struct InvestorTypeWithRange {
    pub investor_type: InvestorType,
    pub min_deposit: u64,
    pub max_deposit: u64,
}

impl InvestorTypeWithRange {
    // Borsh serializes enum variants as 1 byte regardless of #[repr(u16)],
    // so InvestorType is 1 byte on-chain, not 2. repr(u16) only affects
    // in-memory layout for bitwise operations (e.g. `as u16`).
    pub const MAX_SPACE: usize = 1 + 8 + 8; // 1-byte enum variant + u64 + u64
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
pub enum InvestorTier {
    Basic = 1 << 0,
    Accredited = 1 << 1,
    Qualified = 1 << 2,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UserEntry {
    pub user: Pubkey,
    pub investor_type: InvestorType,
    pub investor_tier: InvestorTier,
}

#[account]
pub struct UserWhitelist {
    pub authority: Pubkey,
    pub bump: u8,
    pub users: Vec<UserEntry>,
}

impl UserWhitelist {
    pub const MAX_USERS: usize = 200;
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

    pub fn add_user(
        &mut self,
        user: Pubkey,
        investor_type: InvestorType,
        investor_tier: InvestorTier,
    ) -> Result<()> {
        require!(
            self.users.len() < Self::MAX_USERS,
            UserWhitelistError::WhitelistFull
        );

        // Check if user already exists
        if let Some(existing_user) = self.users.iter_mut().find(|u| u.user == user) {
            // Update existing user's investor type and tier
            existing_user.investor_type = investor_type;
            existing_user.investor_tier = investor_tier;
        } else {
            // Add new user
            self.users.resize(
                self.users.len() + 1,
                UserEntry {
                    user,
                    investor_type,
                    investor_tier,
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

    pub fn get_user_classification(&self, user: &Pubkey) -> Option<(InvestorType, InvestorTier)> {
        self.users
            .iter()
            .find(|u| u.user == *user)
            .map(|u| (u.investor_type, u.investor_tier))
    }

    pub fn is_user_whitelisted(&self, user: &Pubkey) -> bool {
        self.users.iter().any(|u| u.user == *user)
    }
}

impl UserEntry {
    pub const MAX_SPACE: usize = 32 // user pubkey
        + 1  // investor_type (Borsh enum variant index, 1 byte)
        + 1; // investor_tier (Borsh enum variant index, 1 byte)
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
    #[msg("User whitelist has already been migrated to the current format")]
    AlreadyMigrated,
}
