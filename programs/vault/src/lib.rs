use anchor_lang::prelude::*;

declare_id!("2VTk24HFF9DWuDR8Rn3davgBUBFb6qdN8YMSzv7qzoV9");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
