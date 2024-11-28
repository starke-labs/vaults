use anchor_lang::prelude::*;

#[account]
pub struct TokenWhitelist {
    pub authority: Pubkey,
    pub program_authority: Pubkey,
    pub tokens: Vec<Pubkey>,
    pub bump: u8,
}

impl TokenWhitelist {
    pub const MAX_TOKENS: usize = 100;
    pub const MAX_SPACE: usize = 8 + // discriminator
        32 + // authority pubkey
        32 + // program authority pubkey
        4 + // vec length
        (32 * Self::MAX_TOKENS) + // tokens (100 max)
        1; // bump

    pub const SEED: &'static [u8] = b"STARKE_TOKEN_WHITELIST";

    pub fn initialize(
        &mut self,
        authority: Pubkey,
        program_authority: Pubkey,
        bump: u8,
    ) -> Result<()> {
        require!(
            authority == program_authority,
            WhitelistError::UnauthorizedAccess
        );

        self.authority = authority;
        self.program_authority = program_authority;
        self.tokens = Vec::new();
        self.bump = bump;
        Ok(())
    }

    pub fn add_token(&mut self, token: Pubkey) -> Result<()> {
        require!(
            !self.tokens.contains(&token),
            WhitelistError::TokenAlreadyWhitelisted
        );
        require!(
            self.tokens.len() < Self::MAX_TOKENS,
            WhitelistError::WhitelistFull
        );

        self.tokens.push(token);
        Ok(())
    }

    pub fn remove_token(&mut self, token: Pubkey) -> Result<()> {
        if let Some(index) = self.tokens.iter().position(|x| x == &token) {
            self.tokens.remove(index);
            Ok(())
        } else {
            err!(WhitelistError::TokenNotWhitelisted)
        }
    }

    pub fn is_whitelisted(&self, token: &Pubkey) -> bool {
        self.tokens.contains(token)
    }
}

#[error_code]
pub enum WhitelistError {
    #[msg("Token is already whitelisted")]
    TokenAlreadyWhitelisted,
    #[msg("Token is not whitelisted")]
    TokenNotWhitelisted,
    #[msg("Whitelist is full")]
    WhitelistFull,
    #[msg("Only authority can perform this action")]
    UnauthorizedAccess,
    #[msg("Whitelist already initialized")]
    WhitelistAlreadyInitialized,
}
