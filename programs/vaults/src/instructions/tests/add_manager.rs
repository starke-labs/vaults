use super::*;
use crate::{
    constants::STARKE_AUTHORITY,
    instructions::tests::common_utils::{new_default_account, get_default_clock_account},
    program::Vaults,
    state::ManagerWhitelist,
};
use anchor_lang::{AccountDeserialize, AccountSerialize, Id, Result};
use solana_sdk::{account::Account, pubkey::Pubkey, sysvar::clock};

def_default_instruction!(
    get_add_manager_ix,
    pid = Vaults::id(),
    hash = "global:add_manager",
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
    ]
);

/// Verifies successful addition of a manager to the whitelist.
/// Ensures the manager is correctly recorded and the bump seed matches expectations.
#[test]
fn test_add_manager_to_whitelist() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Manager details
    let manager = Pubkey::new_unique();
    let (whitelist_pda, bump) = Pubkey::find_program_address(&[ManagerWhitelist::SEED], &program_id);
    let whitelist_state = ManagerWhitelist {
        managers: vec![],
        bump,
    };

    let whitelist_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_add_manager_ix(STARKE_AUTHORITY, whitelist_pda, &manager)
        .expect("Failed to create add_manager instruction");
    let result = run_ix_expect!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, whitelist_account),
            get_default_clock_account(),
        ]
    );

    // Assertions -------------------------------
    let updated_manager_whitelist_account = result
        .get_account(&whitelist_pda)
        .expect("Failed to get updated whitelist_pda account from Mollusk result");
    let updated_data = updated_manager_whitelist_account.data.clone();
    let updated_state = ManagerWhitelist::try_deserialize(&mut updated_data.as_slice())
        .expect("Failed to deserialize updated ManagerWhitelist account data");
    updated_state
        .managers
        .iter()
        .find(|&t| *t == manager)
        .expect("Added manager not found in manager whitelist");
    assert_eq!(
        updated_state.bump, bump,
        "Bump seed in manager whitelist does not match expected"
    );
}

/// Verifies that unauthorized entities cannot add a manager to the whitelist.
/// Attempts the operation using an invalid authority and expects a failure.
#[test]
fn test_add_manager_unauth_access() {
    let program_id = Vaults::id();

    // Unauthorized owner | Fake authority
    let fake_authority = Pubkey::new_unique();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Manager details
    let manager = Pubkey::new_unique();
    let (whitelist_pda, bump) = Pubkey::find_program_address(&[ManagerWhitelist::SEED], &program_id);
    let whitelist_state = ManagerWhitelist {
        managers: vec![],
        bump,
    };

    let whitelist_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_add_manager_ix(fake_authority, whitelist_pda, &manager)
        .expect("Failed to create add_manager instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (fake_authority, authority_account),
            (whitelist_pda, whitelist_account),
            get_default_clock_account(),
        ]
    );
}

/// Ensures a constraint violation occurs when the bump seed used is incorrect.
/// The operation must fail due to a mismatched bump value.
#[test]
fn test_add_manager_bump_seed_constraint_violation() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Manager details : Ignored bump
    let manager = Pubkey::new_unique();
    let (whitelist_pda, _) = Pubkey::find_program_address(&[ManagerWhitelist::SEED], &program_id);
    let whitelist_state = ManagerWhitelist {
        managers: vec![],
        bump: 0, // Fake bump
    };

    let whitelist_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_add_manager_ix(STARKE_AUTHORITY, whitelist_pda, &manager)
        .expect("Failed to create add_manager instruction");
    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, whitelist_account),
            get_default_clock_account(),
        ]
    );
}

/// Ensures a constraint violation occurs when the manager whitelist address is incorrect.
/// Validates that a mismatched address prevents unauthorized modification.
#[test]
fn test_add_manager_manager_whitelist_address_constraint_violation() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Manager details: ignored pda
    let manager = Pubkey::new_unique();
    let (_, bump) = Pubkey::find_program_address(&[ManagerWhitelist::SEED], &program_id);
    let whitelist_pda = Pubkey::new_unique(); // Fake pda
    let whitelist_state = ManagerWhitelist {
        managers: vec![],
        bump,
    };

    let whitelist_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_add_manager_ix(STARKE_AUTHORITY, whitelist_pda, &manager)
        .expect("Failed to create add_manager instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, whitelist_account),
            get_default_clock_account(),
        ]
    );
}

/// Verifies that a duplicate manager cannot be added to the whitelist.
/// Ensures either an error is thrown or the operation is idempotent.
#[test]
fn test_add_duplicate_manager_to_whitelist() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Manager details
    let manager = Pubkey::new_unique();
    let (whitelist_pda, bump) = Pubkey::find_program_address(&[ManagerWhitelist::SEED], &program_id);
    let whitelist_state = ManagerWhitelist {
        managers: vec![manager],
        bump,
    };

    let whitelist_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_add_manager_ix(STARKE_AUTHORITY, whitelist_pda, &manager)
        .expect("Failed to add_manager(duplicate) instruction");

    let second_result = run_ix!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, whitelist_account,),
            get_default_clock_account(),
        ]
    );

    // Expect the second add to fail OR behave idempotently
    if second_result.raw_result.is_err() {
        println!(
            "Duplicate manager addition correctly failed: {:?}",
            second_result.raw_result
        );
        return;
    }

    // If second call succeeded, ensure no duplicate was added
    let updated_account = second_result
        .get_account(&whitelist_pda)
        .expect("Failed to get updated whitelist_pda account");
    let updated_data = updated_account.data.clone();
    let updated_state = ManagerWhitelist::try_deserialize(&mut updated_data.as_slice())
        .expect("Failed to deserialize updated ManagerWhitelist");

    let matching_managers: Vec<_> = updated_state
        .managers
        .iter()
        .filter(|&&m| m == manager)
        .collect();

    assert_eq!(
        matching_managers.len(),
        1,
        "Duplicate manager found in whitelist: {matching_managers:?}"
    );
}
