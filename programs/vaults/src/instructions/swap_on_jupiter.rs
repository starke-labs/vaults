use anchor_lang::prelude::*;

use crate::state::*;

pub fn _swap_on_jupiter(_ctx: Context<SwapOnJupiter>, _amount: u64) -> Result<()> {
    // TODO: Add swap logic here from:
    //       https://github.com/starke-labs/jup-swap-integration
    Ok(())
}

#[derive(Accounts)]
pub struct SwapOnJupiter<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
}
