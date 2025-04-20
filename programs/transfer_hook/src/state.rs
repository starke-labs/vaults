use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VaultConfig {
    pub key: Pubkey,
    pub manager: Pubkey,
    // Do we need to store the vtoken mint?
    pub vtoken_mint: Pubkey,
    pub vtoken_is_transferrable: bool,
    pub bump: u8,
}

impl VaultConfig {
    pub const SEED: &'static [u8] = b"STARKE_VAULT_CONFIG";
    pub const MAX_SPACE: usize = 8 + // Discriminator
        VaultConfig::INIT_SPACE;

    pub fn initialize(
        &mut self,
        vault_key: &Pubkey,
        manager: &Pubkey,
        vtoken_mint: &Pubkey,
        vtoken_is_transferrable: bool,
        bump: u8,
    ) -> Result<()> {
        self.key = *vault_key;
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
