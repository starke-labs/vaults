use anchor_lang::prelude::*;

#[derive(InitSpace, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct VaultConfig {
    pub key: Pubkey,
    pub manager: Pubkey,
    // TODO: Do we need to store the vtoken mint?
    pub vtoken_mint: Pubkey,
    pub vtoken_is_transferrable: bool,
}

#[account]
#[derive(InitSpace)]
pub struct VaultConfigs {
    #[max_len(100)]
    pub configs: Vec<VaultConfig>,
    pub bump: u8,
}

impl VaultConfigs {
    pub const SEED: &'static [u8] = b"STARKE_VAULT_CONFIG";
    pub const MAX_SPACE: usize = 8 + // Discriminator
        VaultConfigs::INIT_SPACE;

    pub fn initialize(&mut self, bump: u8) -> Result<()> {
        self.bump = bump;
        self.configs = vec![];
        Ok(())
    }

    pub fn add_vault_config(
        &mut self,
        key: &Pubkey,
        manager: &Pubkey,
        vtoken_mint: &Pubkey,
        vtoken_is_transferrable: bool,
        bump: u8,
    ) -> Result<()> {
        self.bump = bump;
        self.configs.push(VaultConfig {
            key: *key,
            manager: *manager,
            vtoken_mint: *vtoken_mint,
            vtoken_is_transferrable,
        });
        Ok(())
    }

    fn get_vault_config(&mut self, vtoken_mint: &Pubkey) -> Result<&mut VaultConfig> {
        self.configs
            .iter_mut()
            .find(|config| &config.vtoken_mint == vtoken_mint)
            .ok_or(error!(VaultConfigsError::VaultConfigNotFound))
    }

    pub fn check_if_vtoken_is_transferrable(&mut self, vtoken_mint: &Pubkey) -> Result<bool> {
        let config = self.get_vault_config(vtoken_mint)?;
        Ok(config.vtoken_is_transferrable)
    }

    pub fn set_vtoken_is_transferrable(
        &mut self,
        vtoken_mint: &Pubkey,
        vtoken_is_transferrable: bool,
    ) -> Result<()> {
        let config = self.get_vault_config(vtoken_mint)?;
        config.vtoken_is_transferrable = vtoken_is_transferrable;
        Ok(())
    }
}

#[error_code]
pub enum VaultConfigsError {
    #[msg("Vault config not found")]
    VaultConfigNotFound,
}
