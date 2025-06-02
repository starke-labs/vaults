use super::*;
use crate::{
    constants::STARKE_AUTHORITY, instructions::tests::common_utils::new_default_account,
    program::Vaults, state::StarkeConfig,
};
use anchor_lang::{AccountDeserialize, AccountSerialize, Id, Result};
use solana_sdk::{account::Account, pubkey::Pubkey};

def_default_instruction!(
    get_pause_starke_ix,
    pid = Vaults::id(),
    hash = "global:pause_starke",
    args = [
        authority_pubkey: Pubkey,
        starke_config_pda: Pubkey,
    ],
    ser_args = [],
    accounts = [
        (authority_pubkey, true),
        (starke_config_pda, false),
    ],
);

/// Tests that the `pause_starke` instruction successfully sets the `is_paused` flag to true
/// when executed by the authorized authority and the bump seed is correct.
#[test]
fn test_pause_starke_success() {
    let program_id = Vaults::id();
    let authority_account = new_default_account(program_id);

    // PDA
    let (starke_config_pda, bump) =
        Pubkey::find_program_address(&[StarkeConfig::SEED], &program_id);

    let starke_config = StarkeConfig {
        is_paused: false,
        bump,
    };

    let starke_config_account = serialize_into_account!(starke_config, program_id);

    let ix = get_pause_starke_ix(STARKE_AUTHORITY, starke_config_pda)
        .expect("Failed to create starke_pause instruction");

    // Instruction ------------------------------
    let result = run_ix_expect!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (starke_config_pda, starke_config_account),
        ]
    );

    // Assertions -------------------------------
    let updated_account = result
        .get_account(&starke_config_pda)
        .expect("Missing starke config account");

    let updated_config = StarkeConfig::try_deserialize(&mut updated_account.data.as_slice())
        .expect("Deserialization failed");

    assert!(updated_config.is_paused, "Expected `is_paused` to be true");
}

/// Ensures that an unauthorized signer cannot pause the protocol.
/// Expects the instruction to fail due to invalid authority.
#[test]
fn test_pause_starke_unauthorized_should_fail() {
    let program_id = Vaults::id();

    // Unauthorized owner | Fake authority
    let fake_authority = Pubkey::new_unique();

    let authority_account = new_default_account(program_id);

    let (starke_config_pda, bump) =
        Pubkey::find_program_address(&[StarkeConfig::SEED], &program_id);

    let starke_config = StarkeConfig {
        is_paused: false,
        bump,
    };

    let starke_config_account = serialize_into_account!(starke_config, program_id);

    // Instruction ------------------------------
    let ix = get_pause_starke_ix(fake_authority, starke_config_pda)
        .expect("Failed to create starke_pause instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (fake_authority, authority_account),
            (starke_config_pda, starke_config_account),
        ]
    );
}

/// Validates that the instruction fails when the bump seed does not match
/// the PDA derived from the program and seed.
/// Expects a constraint violation error.
#[test]
fn test_pause_starke_wrong_bump_should_fail() {
    let program_id = Vaults::id();
    let authority_account = new_default_account(program_id);

    // Correct address but wrong bump
    let (starke_config_pda, _) = Pubkey::find_program_address(&[StarkeConfig::SEED], &program_id);

    let starke_config = StarkeConfig {
        is_paused: false,
        bump: 0, // wrong bump
    };

    let starke_config_account = serialize_into_account!(starke_config, program_id);

    // Instruction ------------------------------
    let ix = get_pause_starke_ix(STARKE_AUTHORITY, starke_config_pda)
        .expect("Failed to create starke_pause instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (starke_config_pda, starke_config_account),
        ]
    );
}
