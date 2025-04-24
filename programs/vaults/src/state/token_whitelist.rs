use anchor_lang::prelude::*;

#[account]
pub struct TokenWhitelist {
    // TODO: Essentially the authority is the same as the program authority, and needs to be removed
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
    pub price_update: Pubkey,
}

impl TokenWhitelist {
    // NOTE: Had to limit this because max space is 10240
    pub const MAX_TOKENS: usize = 75;
    pub const MAX_SPACE: usize = 8 + // discriminator
        32 + // authority pubkey
        32 + // program authority pubkey
        4 + // vec length
        // TokenInfo struct:
        // - 32 for the mint pubkey
        // - 66 for the price feed id (eg: 0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43)
        // - 32 for the price update pubkey
        (32 + 66 + 32) * Self::MAX_TOKENS + // tokens (75 max)
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
            TokenWhitelistError::UnauthorizedAccess
        );

        self.authority = *authority;
        self.program_authority = *program_authority;
        self.tokens = Vec::new();
        self.bump = bump;
        Ok(())
    }

    pub fn add_token(
        &mut self,
        token_mint: &Pubkey,
        price_feed_id: &str,
        price_update: &Pubkey,
    ) -> Result<()> {
        require!(
            !self.tokens.iter().any(|t| t.mint == *token_mint),
            TokenWhitelistError::TokenAlreadyWhitelisted
        );
        require!(
            self.tokens.len() < Self::MAX_TOKENS,
            TokenWhitelistError::WhitelistFull
        );

        self.tokens.push(TokenInfo {
            mint: *token_mint,
            price_feed_id: price_feed_id.to_string(),
            price_update: *price_update,
        });
        Ok(())
    }

    pub fn remove_token(&mut self, token_mint: &Pubkey) -> Result<()> {
        if let Some(index) = self.tokens.iter().position(|t| t.mint == *token_mint) {
            self.tokens.remove(index);
            Ok(())
        } else {
            err!(TokenWhitelistError::TokenNotWhitelisted)
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
            .ok_or(TokenWhitelistError::TokenNotWhitelisted)?;
        Ok(token_info.price_feed_id.as_str())
    }

    pub fn verify_price_update(&self, token_mint: &Pubkey, price_update: &Pubkey) -> Result<()> {
        let token_info = self
            .tokens
            .iter()
            .find(|t| t.mint == *token_mint)
            .ok_or(TokenWhitelistError::TokenNotWhitelisted)?;
        require!(
            token_info.price_update == *price_update,
            TokenWhitelistError::PriceUpdateNotVerified
        );
        Ok(())
    }
}

#[error_code]
pub enum TokenWhitelistError {
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
    #[msg("Price update not verified")]
    PriceUpdateNotVerified,
}
