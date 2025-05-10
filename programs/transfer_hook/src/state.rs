use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VtokenConfig {
    pub manager: Pubkey,
    pub vtoken_mint: Pubkey,
    pub vtoken_is_transferrable: bool,
    pub bump: u8,
}

impl VtokenConfig {
    pub const SEED: &'static [u8] = b"STARKE_VTOKEN_CONFIG";
    pub const MAX_SPACE: usize = 8 + // Discriminator
        VtokenConfig::INIT_SPACE;

    pub fn initialize(
        &mut self,
        manager: &Pubkey,
        vtoken_mint: &Pubkey,
        vtoken_is_transferrable: bool,
        bump: u8,
    ) -> Result<()> {
        self.manager = *manager;
        self.vtoken_mint = *vtoken_mint;
        self.vtoken_is_transferrable = vtoken_is_transferrable;
        self.bump = bump;

        Ok(())
    }

    pub fn set_vtoken_is_transferrable(&mut self, vtoken_is_transferrable: bool) -> Result<()> {
        self.vtoken_is_transferrable = vtoken_is_transferrable;

        Ok(())
    }

    pub fn check_if_vtoken_is_transferrable(&self) -> Result<bool> {
        Ok(self.vtoken_is_transferrable)
    }
}
