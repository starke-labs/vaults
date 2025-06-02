use super::*;
use crate::{
    constants::STARKE_AUTHORITY, instructions::tests::common_utils::new_default_account,
    program::Vaults, run_ix_expect_err, serialize_into_account, state::StarkeConfig,
};
use anchor_lang::{AccountDeserialize, AccountSerialize, Id, Result};
use solana_sdk::{account::Account, pubkey::Pubkey};

def_default_instruction!(
    get_resume_starke_ix,
    pid = Vaults::id(),
    hash = "global:resume_starke",
    args = [
        authority_pubkey: Pubkey,
        starke_config_pubkey: Pubkey,
    ],
    ser_args = [],
    accounts = [
        (authority_pubkey, true),
        (starke_config_pubkey, false),
    ],
);

/// Tests that the resume instruction sets `is_paused = false`.
#[test]
fn test_resume_starke_success() {
    let program_id = Vaults::id();
    let authority_account = new_default_account(program_id);

    // PDA
    let (starke_config_pubkey, bump) =
        Pubkey::find_program_address(&[StarkeConfig::SEED], &program_id);

    let starke_config = StarkeConfig {
        is_paused: true,
        bump,
    };

    let starke_config_account = serialize_into_account!(starke_config, program_id);

    // Instruction ------------------------------
    let ix = get_resume_starke_ix(STARKE_AUTHORITY, starke_config_pubkey)
        .expect("Failed to create starke_resume instruction");
    let result = run_ix_expect!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (starke_config_pubkey, starke_config_account),
        ]
    );

    // Assertions -------------------------------
    let updated_account = result
        .get_account(&starke_config_pubkey)
        .expect("Missing starke config account");

    let updated_config = StarkeConfig::try_deserialize(&mut updated_account.data.as_slice())
        .expect("Deserialization failed");

    assert!(
        !updated_config.is_paused,
        "Expected `is_paused` to be false"
    );
}

/// Tests that a non-authorized signer cannot resume.
#[test]
fn test_resume_starke_unauthorized_should_fail() {
    let program_id = Vaults::id();

    // Unauthorized owner | Fake authority
    let fake_authority = Pubkey::new_unique();

    let authority_account = new_default_account(program_id);

    // PDA
    let (starke_config_pubkey, bump) =
        Pubkey::find_program_address(&[StarkeConfig::SEED], &program_id);

    let starke_config = StarkeConfig {
        is_paused: true,
        bump,
    };

    let starke_config_account = serialize_into_account!(starke_config, program_id);

    // Instruction ------------------------------
    let ix = get_resume_starke_ix(fake_authority, starke_config_pubkey)
        .expect("Failed to create starke_resume instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (fake_authority, authority_account),
            (starke_config_pubkey, starke_config_account),
        ]
    );
}

/// Tests that bump seed mismatch fails.
#[test]
fn test_resume_starke_wrong_bump_should_fail() {
    let program_id = Vaults::id();
    let authority_account = new_default_account(program_id);

    // PDA
    let (starke_config_pubkey, _) =
        Pubkey::find_program_address(&[StarkeConfig::SEED], &program_id);
    let starke_config = StarkeConfig {
        is_paused: true,
        bump: 0, // Fake bump
    };

    let starke_config_account = serialize_into_account!(starke_config, program_id);

    // Instruction ------------------------------
    let ix = get_resume_starke_ix(STARKE_AUTHORITY, starke_config_pubkey)
        .expect("Failed to create starke_resume instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (starke_config_pubkey, starke_config_account),
        ]
    );
}
