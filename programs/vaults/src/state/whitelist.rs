use anchor_lang::prelude::*;

#[account]
pub struct TokenWhitelist {
    pub authority: Pubkey,
    pub program_authority: Pubkey,
    pub tokens: Vec<TokenInfo>,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TokenInfo {
    pub mint: Pubkey,
    // Price feed id from https://www.pyth.network/developers/price-feed-ids#stable
    pub price_feed_id: String,
}

impl TokenWhitelist {
    pub const MAX_TOKENS: usize = 100;
    pub const MAX_SPACE: usize = 8 + // discriminator
        32 + // authority pubkey
        32 + // program authority pubkey
        4 + // vec length
        // TokenInfo struct:
        // - 32 for the mint pubkey
        // - 66 for the price feed id (eg: 0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43)
        (32 + 66) * Self::MAX_TOKENS + // tokens (100 max)
        1; // bump

    pub const SEED: &'static [u8] = b"STARKE_TOKEN_WHITELIST";

    pub fn initialize(
        &mut self,
        authority: &Pubkey,
        program_authority: &Pubkey,
        bump: u8,
    ) -> Result<()> {
        require!(
            authority == program_authority,
            WhitelistError::UnauthorizedAccess
        );

        self.authority = *authority;
        self.program_authority = *program_authority;
        self.tokens = Vec::new();
        self.bump = bump;
        Ok(())
    }

    pub fn add_token(&mut self, token_mint: &Pubkey, price_feed_id: &str) -> Result<()> {
        require!(
            !self.tokens.iter().any(|t| t.mint == *token_mint),
            WhitelistError::TokenAlreadyWhitelisted
        );
        require!(
            self.tokens.len() < Self::MAX_TOKENS,
            WhitelistError::WhitelistFull
        );

        self.tokens.push(TokenInfo {
            mint: *token_mint,
            price_feed_id: price_feed_id.to_string(),
        });
        Ok(())
    }

    pub fn remove_token(&mut self, token_mint: &Pubkey) -> Result<()> {
        if let Some(index) = self.tokens.iter().position(|t| t.mint == *token_mint) {
            self.tokens.remove(index);
            Ok(())
        } else {
            err!(WhitelistError::TokenNotWhitelisted)
        }
    }

    pub fn is_whitelisted(&self, token_mint: &Pubkey) -> bool {
        self.tokens.iter().any(|t| t.mint == *token_mint)
    }

    pub fn get_price_feed_id(&self, token_mint: &Pubkey) -> Result<&str> {
        let token_info = self
            .tokens
            .iter()
            .find(|t| t.mint == *token_mint)
            .ok_or(WhitelistError::TokenNotWhitelisted)?;
        Ok(token_info.price_feed_id.as_str())
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
