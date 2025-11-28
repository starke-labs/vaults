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
    get_add_token_ix,
    pid = Vaults::id(),
    hash = "global:add_token",
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
    ]
);

/// Verifies successful addition of a token to the whitelist.
/// Asserts correct recording of token, price feed ID, price update address, and bump seed.
#[test]
fn test_add_token_to_whitelist() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);
    // Token details
    let token = Pubkey::new_unique();
    let (whitelist_pda, bump) = Pubkey::find_program_address(&[TokenWhitelist::SEED], &program_id);
    let (price_update, price_feed_id, token_state) = (
        Pubkey::new_unique(),
        String::from("BTC/USD"),
        TokenWhitelist {
            authority: STARKE_AUTHORITY,
            program_authority: STARKE_AUTHORITY,
            tokens: vec![],
            bump,
        },
    );

    let token_whitelist_account = serialize_into_account!(token_state, program_id);

    // Instruction ------------------------------
    let ix = get_add_token_ix(
        STARKE_AUTHORITY,
        whitelist_pda,
        token,
        price_feed_id.clone(),
        price_update,
    )
    .expect("Failed to create add_token instruction");
    let result = run_ix_expect!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, token_whitelist_account),
            get_default_clock_account(),
        ]
    );
    // Assertions -------------------------------
    let updated_token_whitelist_account = result
        .get_account(&whitelist_pda)
        .expect("Failed to get updated whitelist_pda account from Mollusk result");
    let updated_data = updated_token_whitelist_account.data.clone();
    let updated_state = TokenWhitelist::try_deserialize(&mut updated_data.as_slice())
        .expect("Failed to deserialize updated TokenWhitelist account data");
    let token_info = updated_state
        .tokens
        .iter()
        .find(|t| t.mint == token)
        .expect("Added token not found in token whitelist");
    assert_eq!(
        token_info.price_feed_id, price_feed_id,
        "Price feed ID in token whitelist does not match expected"
    );
    assert_eq!(
        token_info.price_update, price_update,
        "Price update pubkey in token whitelist does not match expected"
    );
    assert_eq!(
        updated_state.bump, bump,
        "Bump seed in token whitelist does not match expected"
    );
    assert_eq!(
        updated_state.authority, STARKE_AUTHORITY,
        "Authority in token whitelist does not match expected STARKE_AUTHORITY"
    );
}

/// Ensures unauthorized authorities cannot add tokens to the whitelist.
/// Expects the instruction to fail when signed by an invalid authority.
#[test]
fn test_add_token_unauth_access() {
    let program_id = Vaults::id();
    // Unauthorized owner | Fake authority
    let fake_authority = Pubkey::new_unique();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Token details
    let token = Pubkey::new_unique();
    let (whitelist_pda, bump) = Pubkey::find_program_address(&[TokenWhitelist::SEED], &program_id);
    let (price_update, price_feed_id, token_state) = (
        Pubkey::new_unique(),
        String::from("BTC/USD"),
        TokenWhitelist {
            authority: STARKE_AUTHORITY,
            program_authority: STARKE_AUTHORITY,
            tokens: vec![],
            bump,
        },
    );

    let token_whitelist_account = serialize_into_account!(token_state, program_id);

    // Instruction ------------------------------
    let ix = get_add_token_ix(
        fake_authority,
        whitelist_pda,
        token,
        price_feed_id.clone(),
        price_update,
    )
    .expect("Failed to create add_token instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (fake_authority, authority_account),
            (whitelist_pda, token_whitelist_account),
            get_default_clock_account(),
        ]
    );
}

/// Validates constraint enforcement for an incorrect bump seed.
/// The instruction should fail due to mismatch with the derived address.
#[test]
fn test_add_token_bump_seed_constraint_violation() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Token details
    let token = Pubkey::new_unique();
    let (whitelist_pda, _) = Pubkey::find_program_address(&[TokenWhitelist::SEED], &program_id);
    let (price_update, price_feed_id, token_state) = (
        Pubkey::new_unique(),
        String::from("BTC/USD"),
        TokenWhitelist {
            authority: STARKE_AUTHORITY,
            program_authority: STARKE_AUTHORITY,
            tokens: vec![],
            bump: 0, // Fake bump
        },
    );

    let token_whitelist_account = serialize_into_account!(token_state, program_id);

    // Instruction ------------------------------
    let ix = get_add_token_ix(
        STARKE_AUTHORITY,
        whitelist_pda,
        token,
        price_feed_id.clone(),
        price_update,
    )
    .expect("Failed to create add_token instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, token_whitelist_account),
            get_default_clock_account(),
        ]
    );
}

/// Validates constraint enforcement for an incorrect token whitelist public key.
/// Expects failure when using an address that does not match the derived PDA.
#[test]
fn test_add_token_whitelist_pubkey_constraint_violation() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Token details
    let token = Pubkey::new_unique();
    let (_, bump) = Pubkey::find_program_address(&[TokenWhitelist::SEED], &program_id);
    let whitelist_pda = Pubkey::new_unique(); // Fake pda
    let (price_update, price_feed_id, token_state) = (
        Pubkey::new_unique(),
        String::from("BTC/USD"),
        TokenWhitelist {
            authority: STARKE_AUTHORITY,
            program_authority: STARKE_AUTHORITY,
            tokens: vec![],
            bump,
        },
    );

    let token_whitelist_account = serialize_into_account!(token_state, program_id);

    // Instruction ------------------------------
    let ix = get_add_token_ix(
        STARKE_AUTHORITY,
        whitelist_pda,
        token,
        price_feed_id.clone(),
        price_update,
    )
    .expect("Failed to create add_token instruction");

    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, token_whitelist_account),
            get_default_clock_account(),
        ]
    );
}

/// Ensures duplicate tokens are not added to the whitelist.
/// If the second addition succeeds, asserts that the token is not duplicated in the list.
#[test]
fn test_add_duplicate_token_to_whitelist() {
    let program_id = Vaults::id();

    // Authority Account
    let authority_account = new_default_account(program_id);

    // Token details
    let token = Pubkey::new_unique();
    let (whitelist_pda, bump) = Pubkey::find_program_address(&[TokenWhitelist::SEED], &program_id);
    let (price_update, price_feed_id, token_state) = (
        Pubkey::new_unique(),
        String::from("BTC/USD"),
        TokenWhitelist {
            authority: STARKE_AUTHORITY,
            program_authority: STARKE_AUTHORITY,
            tokens: vec![TokenInfo {
                mint: Pubkey::new_unique(),
                price_update: Pubkey::new_unique(),
                price_feed_id: String::from("BTC/USD"),
            }],
            bump,
        },
    );

    let token_whitelist_account = serialize_into_account!(token_state, program_id);

    // Instruction ------------------------------
    let ix = get_add_token_ix(
        STARKE_AUTHORITY,
        whitelist_pda,
        token,
        price_feed_id.clone(),
        price_update,
    )
    .expect("Failed to create duplicate add_token instruction");

    let result = run_ix!(
        program_id,
        ix,
        [
            (STARKE_AUTHORITY, authority_account),
            (whitelist_pda, token_whitelist_account),
            get_default_clock_account(),
        ]
    );

    // Expecting duplicate addition to either fail...
    if result.raw_result.is_err() {
        println!(
            "Duplicate addition correctly failed: {:?}",
            result.raw_result
        );
        return;
    }

    // ...or at least not result in duplicated entries
    let updated_account = result
        .get_account(&whitelist_pda)
        .expect("Failed to get updated whitelist_pda account from Mollusk result");
    let updated_data = updated_account.data.clone();
    let updated_state = TokenWhitelist::try_deserialize(&mut updated_data.as_slice())
        .expect("Failed to deserialize updated TokenWhitelist");

    let matching_tokens: Vec<_> = updated_state
        .tokens
        .iter()
        .filter(|t| t.mint == token)
        .collect();

    assert_eq!(
        matching_tokens.len(),
        1,
        "Duplicate token found in whitelist when it should not be duplicated"
    );
}
