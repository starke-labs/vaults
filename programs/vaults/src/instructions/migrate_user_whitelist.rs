use anchor_lang::prelude::*;

use crate::constants::STARKE_AUTHORITY;
use crate::state::{UserEntry, UserWhitelist, UserWhitelistError};

/// Size of a UserEntry in the old format (without investor_tier)
const OLD_USER_ENTRY_SIZE: usize = 32 // user pubkey
    + 1; // investor_type enum (u8)

/// Total allocated space of UserWhitelist in the old format
const OLD_MAX_SPACE: usize = 8  // discriminator
    + 32                         // authority
    + 1                          // bump
    + 4                          // users vec length
    + OLD_USER_ENTRY_SIZE * UserWhitelist::MAX_USERS;

/// Byte offset where user entries begin in serialized UserWhitelist data
const ENTRIES_OFFSET: usize = 8 + 32 + 1 + 4;

pub fn _migrate_user_whitelist(ctx: Context<MigrateUserWhitelist>) -> Result<()> {
    let account_info = ctx.accounts.user_whitelist.to_account_info();

    // Parse old-format data before realloc (borrow dropped at end of block)
    let (discriminator, authority_bytes, bump, users_len, old_entries) = {
        let data = account_info.data.borrow();

        require!(
            data.len() == OLD_MAX_SPACE,
            UserWhitelistError::AlreadyMigrated
        );

        let discriminator: [u8; 8] = data[..8].try_into().unwrap();
        let authority_bytes: [u8; 32] = data[8..40].try_into().unwrap();
        let bump = data[40];
        let users_len = u32::from_le_bytes(data[41..45].try_into().unwrap()) as usize;

        require!(
            users_len <= UserWhitelist::MAX_USERS,
            UserWhitelistError::AlreadyMigrated
        );

        let mut old_entries: Vec<([u8; 32], u8, u8)> = Vec::with_capacity(users_len);
        for i in 0..users_len {
            let base = ENTRIES_OFFSET + i * OLD_USER_ENTRY_SIZE;
            let user_bytes: [u8; 32] = data[base..base + 32].try_into().unwrap();
            let old_investor_type_byte = data[base + 32];
            // IMPORTANT: This mapping assumes on-chain data uses the original 4-variant
            // InvestorType enum (Retail=0, Accredited=1, Institutional=2, Qualified=3).
            // An intermediate refactor introduced a 2-variant enum (Entity=0, Individual=1)
            // but it was never deployed to write user entries, so this mapping is safe.
            //
            // Map old flat InvestorType to new split (InvestorType, InvestorTier)
            // using Borsh variant indices:
            //   New InvestorType: Unknown=0, Entity=1, Individual=2
            //   New InvestorTier: Basic=0, Accredited=1, Qualified=2
            let (new_type_byte, new_tier_byte) = match old_investor_type_byte {
                0 => (2, 0), // Retail       -> Individual + Basic
                1 => (2, 1), // Accredited   -> Individual + Accredited
                2 => (1, 0), // Institutional -> Entity + Basic
                3 => (2, 2), // Qualified    -> Individual + Qualified
                _ => return err!(UserWhitelistError::InvalidInvestorType),
            };
            old_entries.push((user_bytes, new_type_byte, new_tier_byte));
        }

        (discriminator, authority_bytes, bump, users_len, old_entries)
    };

    // Fund rent difference if the larger allocation requires more lamports
    let new_minimum_balance = Rent::get()?.minimum_balance(UserWhitelist::MAX_SPACE);
    let current_lamports = account_info.lamports();
    if current_lamports < new_minimum_balance {
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.starke_authority.to_account_info(),
                    to: account_info.clone(),
                },
            ),
            new_minimum_balance - current_lamports,
        )?;
    }

    account_info.realloc(UserWhitelist::MAX_SPACE, false)?;

    // Write new-format data with remapped InvestorType and derived InvestorTier
    {
        let mut data = account_info.data.borrow_mut();
        data[..8].copy_from_slice(&discriminator);
        data[8..40].copy_from_slice(&authority_bytes);
        data[40] = bump;
        data[41..45].copy_from_slice(&(users_len as u32).to_le_bytes());

        for (i, (user_bytes, type_byte, tier_byte)) in old_entries.iter().enumerate() {
            let base = ENTRIES_OFFSET + i * UserEntry::MAX_SPACE;
            data[base..base + 32].copy_from_slice(user_bytes);
            data[base + 32] = *type_byte;
            data[base + 33] = *tier_byte;
        }
    }

    msg!(
        "Migrated {} users to new format with InvestorTier::Basic as default",
        users_len
    );
    Ok(())
}

#[derive(Accounts)]
pub struct MigrateUserWhitelist<'info> {
    #[account(mut, address = STARKE_AUTHORITY @ UserWhitelistError::UnauthorizedAccess)]
    pub starke_authority: Signer<'info>,

    /// CHECK: Manual deserialization for migration from old format (without investor_tier field)
    #[account(
        mut,
        seeds = [UserWhitelist::SEED],
        bump,
    )]
    pub user_whitelist: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
