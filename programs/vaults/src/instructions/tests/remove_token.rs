use super::*;
use crate::{
    constants::STARKE_AUTHORITY,
    instructions::tests::common_utils::{get_default_clock_account, new_default_account},
    program::Vaults,
    state::{TokenInfo, TokenWhitelist},
};
use anchor_lang::{AccountDeserialize, AccountSerialize, Id, Result};
use solana_sdk::{account::Account, pubkey::Pubkey, sysvar::clock};

def_default_instruction!(
    get_remove_token_ix,
    pid = Vaults::id(),
    hash = "global:remove_token",
    args = [
        authority_pubkey: Pubkey,
        token_whitelist_pubkey: Pubkey,
    ],
    ser_args = [
        token: Pubkey => token.to_bytes(),
        price_feed_id: String => {
            let b = price_feed_id.as_bytes();
            [&(b.len() as u32).to_le_bytes(), b].concat()
        },
        price_update: Pubkey => price_update.to_bytes(),
    ],
    accounts = [
        (authority_pubkey, true),
        (token_whitelist_pubkey, false),
        (clock::id(), false),
    ],
);

/// Ensures a manager is correctly removed from the whitelist.
#[test]
fn test_remove_token_from_whitelist() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Token details
    let token = Pubkey::new_unique();
    let (whitelist_pda, bump) = Pubkey::find_program_address(&[TokenWhitelist::SEED], &program_id);
    let (price_update, price_feed_id) = (Pubkey::new_unique(), String::from("BTC/USD"));

    // Initial whitelist state: token is already included
    let token_info = TokenInfo {
        mint: token,
        price_feed_id: price_feed_id.clone(),
        price_update,
    };

    let whitelist_state = TokenWhitelist {
        authority: STARKE_AUTHORITY,
        program_authority: STARKE_AUTHORITY,
        tokens: vec![token_info],
        bump,
    };

    let serialize_into_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_remove_token_ix(
        STARKE_AUTHORITY,
        whitelist_pda,
        token,
        price_feed_id.clone(),
        price_update,
    )
    .expect("Failed to create remove_token instruction");
    let result = run_ix_expect!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, serialize_into_account),
            get_default_clock_account(),
        ]
    );

    // Assertions -------------------------------
    let updated_token_whitelist_account = result
        .get_account(&whitelist_pda)
        .expect("Failed to get updated whitelist_pda account");
    let updated_state =
        TokenWhitelist::try_deserialize(&mut updated_token_whitelist_account.data.as_slice())
            .expect("Failed to deserialize updated TokenWhitelist");

    // Token should no longer be in the whitelist
    assert!(
        !updated_state.tokens.iter().any(|t| t.mint == token),
        "Token was not removed from whitelist"
    );

    // Sanity checks
    assert_eq!(
        updated_state.bump, bump,
        "Bump does not match original bump"
    );
    assert_eq!(
        updated_state.authority, STARKE_AUTHORITY,
        "Authority mismatch after removal"
    );
}

/// Fails if an unauthorized entity attempts to remove a manager.
#[test]
fn test_remove_token_unauth_access() {
    let program_id = Vaults::id();

    // Unauthorized owner | Fake authority
    let fake_authority = Pubkey::new_unique();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Token details
    let token = Pubkey::new_unique();
    let (whitelist_pda, bump) = Pubkey::find_program_address(&[TokenWhitelist::SEED], &program_id);
    let (price_update, price_feed_id) = (Pubkey::new_unique(), String::from("BTC/USD"));

    // Initial whitelist state: token is already included
    let token_info = TokenInfo {
        mint: token,
        price_feed_id: price_feed_id.clone(),
        price_update,
    };

    let whitelist_state = TokenWhitelist {
        authority: STARKE_AUTHORITY,
        program_authority: STARKE_AUTHORITY,
        tokens: vec![token_info],
        bump,
    };

    let serialize_into_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_remove_token_ix(
        fake_authority,
        whitelist_pda,
        token,
        price_feed_id.clone(),
        price_update,
    )
    .expect("Failed to create remove_token instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (fake_authority, authority_account),
            (whitelist_pda, serialize_into_account),
            get_default_clock_account(),
        ]
    );
}

/// Fails if the bump seed used for the whitelist PDA is incorrect.
#[test]
fn test_remove_token_bump_seed_constraint_violation() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Token details : Ignored bump
    let token = Pubkey::new_unique();
    let (whitelist_pda, _) = Pubkey::find_program_address(&[TokenWhitelist::SEED], &program_id);
    let (price_update, price_feed_id) = (Pubkey::new_unique(), String::from("BTC/USD"));

    // Initial whitelist state: token is already included
    let token_info = TokenInfo {
        mint: token,
        price_feed_id: price_feed_id.clone(),
        price_update,
    };

    let whitelist_state = TokenWhitelist {
        authority: STARKE_AUTHORITY,
        program_authority: STARKE_AUTHORITY,
        tokens: vec![token_info],
        bump: 0, // Fake bump
    };

    let serialize_into_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_remove_token_ix(
        STARKE_AUTHORITY,
        whitelist_pda,
        token,
        price_feed_id.clone(),
        price_update,
    )
    .expect("Failed to create remove_token instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, serialize_into_account),
            get_default_clock_account(),
        ]
    );
}

/// Fails if the whitelist PDA does not match the expected derived address.
#[test]
fn test_remove_token_whitelist_pubkey_constraint_violation() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Token details
    let token = Pubkey::new_unique();
    let (_, bump) = Pubkey::find_program_address(&[TokenWhitelist::SEED], &program_id);
    let (price_update, price_feed_id, whitelist_pda) = (
        Pubkey::new_unique(),
        String::from("BTC/USD"),
        Pubkey::new_unique(), // Fake whitelist_pda
    );

    // Initial whitelist state: token is already included
    let token_info = TokenInfo {
        mint: token,
        price_feed_id: price_feed_id.clone(),
        price_update,
    };

    let whitelist_state = TokenWhitelist {
        authority: STARKE_AUTHORITY,
        program_authority: STARKE_AUTHORITY,
        tokens: vec![token_info],
        bump,
    };

    let serialize_into_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_remove_token_ix(
        STARKE_AUTHORITY,
        whitelist_pda,
        token,
        price_feed_id.clone(),
        price_update,
    )
    .expect("Failed to create remove_token instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, serialize_into_account),
            get_default_clock_account(),
        ]
    );
}

/// Fails when removing a token from an empty whitelist.
#[test]
fn test_remove_token_from_empty_whitelist_should_fail() {
    let program_id = Vaults::id();
    let authority_account = new_default_account(program_id);

    let token = Pubkey::new_unique();
    let (whitelist_pda, bump) = Pubkey::find_program_address(&[TokenWhitelist::SEED], &program_id);
    let (price_update, price_feed_id, whitelist_state) = (
        Pubkey::new_unique(),
        String::from("BTC/USD"),
        TokenWhitelist {
            authority: STARKE_AUTHORITY,
            program_authority: STARKE_AUTHORITY,
            tokens: vec![],
            bump,
        },
    );

    let serialize_into_account = serialize_into_account!(whitelist_state, program_id);

    // Instruction ------------------------------
    let ix = get_remove_token_ix(
        STARKE_AUTHORITY,
        whitelist_pda,
        token,
        price_feed_id.clone(),
        price_update,
    )
    .expect("Failed to create instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, serialize_into_account),
            get_default_clock_account(),
        ]
    );
}

/// Fails when trying to remove a token that is not in the whitelist.
#[test]
fn test_remove_nonexistent_token_should_fail() {
    let program_id = Vaults::id();
    let authority_account = new_default_account(program_id);

    let existing_token = Pubkey::new_unique();
    let (whitelist_pda, bump) = Pubkey::find_program_address(&[TokenWhitelist::SEED], &program_id);
    let (price_update, price_feed_id) = (Pubkey::new_unique(), String::from("BTC/USD"));

    let whitelist_state = TokenWhitelist {
        authority: STARKE_AUTHORITY,
        program_authority: STARKE_AUTHORITY,
        tokens: vec![TokenInfo {
            mint: existing_token,
            price_feed_id: price_feed_id.clone(),
            price_update,
        }],
        bump,
    };

    let serialize_into_account = serialize_into_account!(whitelist_state, program_id);

    // Try to remove a different token
    let non_existent_token = Pubkey::new_unique();

    // Instruction ------------------------------
    let ix = get_remove_token_ix(
        STARKE_AUTHORITY,
        whitelist_pda,
        non_existent_token,
        price_feed_id,
        price_update,
    )
    .expect("Failed to create instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, serialize_into_account),
            get_default_clock_account(),
        ]
    );
}
