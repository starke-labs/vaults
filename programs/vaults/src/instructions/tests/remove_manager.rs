use super::*;
use crate::{
    constants::STARKE_AUTHORITY,
    instructions::tests::common_utils::{get_default_clock_account, new_default_account},
    program::Vaults,
    state::ManagerWhitelist,
};
use anchor_lang::{AccountDeserialize, AccountSerialize, Id, Result};
use solana_sdk::{account::Account, pubkey::Pubkey, sysvar::clock};

def_default_instruction!(
    get_remove_manager_ix,
    pid = Vaults::id(),
    hash = "global:remove_manager",
    args = [
        authority_pubkey: Pubkey,
        manager_whitelist_pubkey: Pubkey,
    ],
    ser_args = [
        manager_pubkey: &Pubkey => manager_pubkey.to_bytes(),
    ],
    accounts = [
        (authority_pubkey, true),
        (manager_whitelist_pubkey, false),
        (clock::id(), false),
    ],
);

/// Tests successful removal of a manager from the whitelist by the authorized authority.
/// Ensures the manager is no longer in the list and bump remains unchanged.
#[test]
fn test_remove_manager_from_whitelist() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Manager details
    let manager = Pubkey::new_unique();
    let (whitelist_pda, bump) =
        Pubkey::find_program_address(&[ManagerWhitelist::SEED], &program_id);

    // Initial whitelist state: manager is already included
    let whitelist_state = ManagerWhitelist {
        managers: vec![manager],
        bump,
    };

    let manager_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_remove_manager_ix(STARKE_AUTHORITY, whitelist_pda, &manager)
        .expect("Failed to create remove_manager instruction");
    let result = run_ix_expect!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, manager_account),
            get_default_clock_account(),
        ]
    );

    // Assertions -------------------------------
    let updated_manager_whitelist_account = result
        .get_account(&whitelist_pda)
        .expect("Failed to get updated whitelist_pda account");

    let updated_state =
        ManagerWhitelist::try_deserialize(&mut updated_manager_whitelist_account.data.as_slice())
            .expect("Failed to deserialize updated ManagerWhitelist");

    // Manager should no longer be in the whitelist
    assert!(
        !updated_state.managers.contains(&manager),
        "Manager was not removed from whitelist"
    );

    // Sanity check
    assert_eq!(
        updated_state.bump, bump,
        "Bump does not match original bump"
    );
}

/// Ensures that an unauthorized signer cannot remove a manager from the whitelist.
/// Expects a constraint failure due to incorrect authority.
#[test]
fn test_remove_manager_unauth_access() {
    let program_id = Vaults::id();

    // Unauthorized owner | Fake authority
    let fake_authority = Pubkey::new_unique();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Manager details
    let manager = Pubkey::new_unique();
    let (whitelist_pda, bump) =
        Pubkey::find_program_address(&[ManagerWhitelist::SEED], &program_id);

    // Initial whitelist state: manager is already included
    let whitelist_state = ManagerWhitelist {
        managers: vec![manager],
        bump,
    };

    let manager_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_remove_manager_ix(fake_authority, whitelist_pda, &manager)
        .expect("Failed to create remove_manager instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (fake_authority, authority_account),
            (whitelist_pda, manager_account),
            get_default_clock_account(),
        ]
    );
}

/// Validates failure when the bump seed provided in the `ManagerWhitelist` account
/// does not match the actual derived bump.
/// Expects a constraint violation.
#[test]
fn test_remove_manager_bump_seed_constraint_violation() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Manager details : bump ignored
    let manager = Pubkey::new_unique();
    let (whitelist_pda, _) = Pubkey::find_program_address(&[ManagerWhitelist::SEED], &program_id);

    // Initial whitelist state: manager is already included
    let whitelist_state = ManagerWhitelist {
        managers: vec![manager],
        bump: 0,
    };

    let manager_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_remove_manager_ix(STARKE_AUTHORITY, whitelist_pda, &manager)
        .expect("Failed to create remove_manager instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, manager_account),
            get_default_clock_account(),
        ]
    );
}

/// Checks failure when the `ManagerWhitelist` account pubkey does not match the PDA
/// derived using the correct seed. Ensures account integrity is enforced.
#[test]
fn test_remove_manager_whitelist_pubkey_constraint_violation() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Manager details
    let manager = Pubkey::new_unique();
    let (_, bump) = Pubkey::find_program_address(&[ManagerWhitelist::SEED], &program_id);
    let whitelist_pda = Pubkey::new_unique(); // Instead have a fake pubkey

    // Initial whitelist state: manager is already included
    let whitelist_state = ManagerWhitelist {
        managers: vec![manager],
        bump,
    };

    let manager_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_remove_manager_ix(STARKE_AUTHORITY, whitelist_pda, &manager)
        .expect("Failed to create remove_manager instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, manager_account),
            get_default_clock_account(),
        ]
    );
}

/// Ensures that trying to remove a manager from an empty whitelist fails gracefully.
/// Expects an error due to the manager not being found.
#[test]
fn test_remove_manager_from_empty_whitelist_should_fail() {
    let program_id = Vaults::id();
    let authority_account = new_default_account(program_id);

    // Manager details
    let manager = Pubkey::new_unique();
    let (whitelist_pda, bump) =
        Pubkey::find_program_address(&[ManagerWhitelist::SEED], &program_id);

    // Whitelist with no managers
    let whitelist_state = ManagerWhitelist {
        managers: vec![],
        bump,
    };

    // Account to whitelist
    let manager_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction
    let ix = get_remove_manager_ix(STARKE_AUTHORITY, whitelist_pda, &manager)
        .expect("Failed to create instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, manager_account),
            get_default_clock_account(),
        ]
    );
}

/// Ensures failure when attempting to remove a manager who is not present in the whitelist.
/// Expects a logic error due to missing entry.
#[test]
fn test_remove_nonexistent_manager_should_fail() {
    let program_id = Vaults::id();
    let authority_account = new_default_account(program_id);

    let existing_manager = Pubkey::new_unique();
    let (whitelist_pda, bump) =
        Pubkey::find_program_address(&[ManagerWhitelist::SEED], &program_id);

    // Add the existing_manager to the whitelist
    let whitelist_state = ManagerWhitelist {
        managers: vec![existing_manager],
        bump,
    };

    // Whitelist account
    let manager_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction
    let ix = get_remove_manager_ix(
        STARKE_AUTHORITY,
        whitelist_pda,
        &Pubkey::new_unique(), // Removing non-existent manager
    )
    .expect("Failed to create instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, manager_account),
            get_default_clock_account(),
        ]
    );
}
