use anchor_lang::prelude::*;

#[account]
pub struct TokenOracleConfig {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub provider: OracleProvider,
    pub pyth_pro_price_feed_id: u32,
    pub pyth_pro_channel_id: u8,
    pub max_age_seconds: u64,
    pub confidence_threshold_bps: u64,
    pub is_active: bool,
    pub updated_at: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OracleProvider {
    PythPro,
}

impl TokenOracleConfig {
    pub const SEED: &'static [u8] = b"STARKE_TOKEN_ORACLE_CONFIG";
    pub const MAX_SPACE: usize = 8 + // discriminator
        32 + // authority
        32 + // mint
        1 + // provider
        4 + // pyth_pro_price_feed_id
        1 + // pyth_pro_channel_id
        8 + // max_age_seconds
        8 + // confidence_threshold_bps
        1 + // is_active
        8 + // updated_at
        1; // bump

    pub fn set_pyth_pro(
        &mut self,
        authority: Pubkey,
        mint: Pubkey,
        price_feed_id: u32,
        channel_id: u8,
        max_age_seconds: u64,
        confidence_threshold_bps: u64,
        is_active: bool,
        updated_at: i64,
        bump: u8,
    ) -> Result<()> {
        require!(
            price_feed_id > 0,
            TokenOracleConfigError::InvalidPriceFeedId
        );
        require!(channel_id > 0, TokenOracleConfigError::InvalidChannelId);
        require!(max_age_seconds > 0, TokenOracleConfigError::InvalidMaxAge);
        require!(
            confidence_threshold_bps > 0,
            TokenOracleConfigError::InvalidConfidenceThreshold
        );

        self.authority = authority;
        self.mint = mint;
        self.provider = OracleProvider::PythPro;
        self.pyth_pro_price_feed_id = price_feed_id;
        self.pyth_pro_channel_id = channel_id;
        self.max_age_seconds = max_age_seconds;
        self.confidence_threshold_bps = confidence_threshold_bps;
        self.is_active = is_active;
        self.updated_at = updated_at;
        self.bump = bump;
        Ok(())
    }

    pub fn verify_pyth_pro(&self, mint: &Pubkey) -> Result<()> {
        require!(self.mint == *mint, TokenOracleConfigError::MintMismatch);
        require!(
            self.provider == OracleProvider::PythPro,
            TokenOracleConfigError::InvalidOracleProvider
        );
        require!(self.is_active, TokenOracleConfigError::OracleConfigInactive);
        Ok(())
    }
}

#[error_code]
pub enum TokenOracleConfigError {
    #[msg("Invalid Pyth Pro price feed id")]
    InvalidPriceFeedId,
    #[msg("Invalid Pyth Pro channel id")]
    InvalidChannelId,
    #[msg("Invalid oracle max age")]
    InvalidMaxAge,
    #[msg("Invalid oracle confidence threshold")]
    InvalidConfidenceThreshold,
    #[msg("Token oracle config mint mismatch")]
    MintMismatch,
    #[msg("Invalid oracle provider")]
    InvalidOracleProvider,
    #[msg("Invalid token oracle config PDA")]
    InvalidOracleConfigPda,
    #[msg("Token oracle config is inactive")]
    OracleConfigInactive,
}
