use anchor_lang::prelude::*;

#[account]
pub struct StarkeConfig {
    pub bump: u8,
    pub is_paused: bool,
}

impl StarkeConfig {
    pub const MAX_SPACE: usize = 8 + 1 + 1;

    pub const SEED: &'static [u8] = b"STARKE_CONFIG";

    pub fn initialize(&mut self, bump: u8) -> Result<()> {
        self.bump = bump;
        self.is_paused = false;
        Ok(())
    }

    pub fn update_pause_status(&mut self, is_paused: bool) -> Result<()> {
        self.is_paused = is_paused;
        Ok(())
    }
}

#[error_code]
pub enum StarkeConfigError {
    #[msg("Unauthorized")]
    Unauthorized,
    // App server relies on this log message (DON'T CHANGE OR REMOVE)
    #[msg("Starke is paused")]
    StarkePaused,
}
