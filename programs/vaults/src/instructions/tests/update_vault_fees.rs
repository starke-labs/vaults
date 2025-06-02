use super::*;
use crate::{
    constants::STARKE_AUTHORITY,
    instructions::tests::common_utils::{get_default_clock_account, new_default_account},
    program::Vaults,
    state::Vault,
};
use anchor_lang::{AccountDeserialize, AccountSerialize, Id, Result};
use solana_sdk::{account::Account, pubkey::Pubkey, sysvar::clock};

def_default_instruction!(
    get_update_vault_fees_ix,
    pid = Vaults::id(),
    hash = "global:update_vault_fees",
    args = [
        manager_pubkey: Pubkey,
        vault_pubkey: Pubkey,
        clock_pubkey: Pubkey,
    ],
    ser_args = [
        new_entry_fee: u16 => new_entry_fee.to_le_bytes(),
        new_exit_fee: u16 => new_exit_fee.to_le_bytes(),
    ],
    accounts = [
        (manager_pubkey, true),
        (vault_pubkey, false),
        (clock_pubkey, false),
    ],
);

#[test]
fn test_update_vault_fees_success() {
    let program_id = Vaults::id();
    let manager_account = new_default_account(program_id);
    let clock_account = get_default_clock_account();

    let manager_pubkey = STARKE_AUTHORITY;
    let (vault_pubkey, bump) =
        Pubkey::find_program_address(&[Vault::SEED, manager_pubkey.as_ref()], &program_id);

    let vault_state = Vault {
        manager: Default::default(),
        deposit_token_mint: Pubkey::new_unique(),
        name: String::from("Vault1"),
        bump,
        mint: Default::default(),
        mint_bump: 0,
        entry_fee: 0,
        exit_fee: 0,
        pending_entry_fee: None,
        pending_exit_fee: None,
        fee_update_timestamp: 0,
        min_deposit_amount: None,
        is_private_vault: false,
        max_allowed_aum: None,
    };
    let vault_account = serialize_into_account!(vault_state, program_id);

    // Updates values
    let (new_entry_fee, new_exit_fee) = (75, 30);

    // Instruction ------------------------------
    let ix = get_update_vault_fees_ix(
        manager_pubkey,
        vault_pubkey,
        clock::id(),
        new_entry_fee,
        new_exit_fee,
    )
    .expect("Failed to create ix");
    let result = run_ix_expect!(
        program_id,
        ix,
        vec![
            (manager_pubkey, manager_account),
            (vault_pubkey, vault_account),
            clock_account,
        ]
    );

    // Assertions -------------------------------
    let updated_vault = result
        .get_account(&vault_pubkey)
        .expect("Missing updated vault account");

    let vault_state =
        Vault::try_deserialize(&mut updated_vault.data.as_slice()).expect("Deserialization failed");

    assert_eq!(vault_state.pending_entry_fee, Some(new_entry_fee));
    assert_eq!(vault_state.pending_exit_fee, Some(new_exit_fee));
}

#[test]
fn test_update_vault_fees_unauthorized_should_fail() {
    let program_id = Vaults::id();
    let fake_manager = Pubkey::new_unique();
    let fake_manager_account = new_default_account(program_id);

    let (vault_pubkey, bump) =
        Pubkey::find_program_address(&[Vault::SEED, STARKE_AUTHORITY.as_ref()], &program_id);

    let vault_state = Vault {
        manager: Default::default(),
        deposit_token_mint: Pubkey::new_unique(),
        name: String::from("Vault1"),
        bump,
        mint: Default::default(),
        mint_bump: 0,
        entry_fee: 0,
        exit_fee: 0,
        pending_entry_fee: None,
        pending_exit_fee: None,
        fee_update_timestamp: 0,
        min_deposit_amount: None,
        is_private_vault: false,
        max_allowed_aum: None,
    };
    let vault_account = serialize_into_account!(vault_state, program_id);

    // Instruction ------------------------------
    let ix = get_update_vault_fees_ix(fake_manager, vault_pubkey, clock::id(), 100, 100)
        .expect("Failed to create ix");
    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (fake_manager, fake_manager_account),
            (vault_pubkey, vault_account),
            get_default_clock_account(),
        ]
    );
}

#[test]
fn test_update_vault_fees_seed_constraint_violation_should_fail() {
    let program_id = Vaults::id();
    let manager_account = new_default_account(program_id);
    let fake_vault_pubkey = Pubkey::new_unique(); // Invalid PDA

    let vault_state = Vault {
        manager: Default::default(),
        deposit_token_mint: Pubkey::new_unique(),
        name: String::from("Vault1"),
        bump: 0,
        mint: Default::default(),
        mint_bump: 0,
        entry_fee: 0,
        exit_fee: 0,
        pending_entry_fee: None,
        pending_exit_fee: None,
        fee_update_timestamp: 0,
        min_deposit_amount: None,
        is_private_vault: false,
        max_allowed_aum: None,
    };
    let vault_account = serialize_into_account!(vault_state, program_id);

    // Instruction ------------------------------
    let ix = get_update_vault_fees_ix(STARKE_AUTHORITY, fake_vault_pubkey, clock::id(), 100, 100)
        .expect("Failed to create ix");
    run_ix_expect_err!(
        program_id,
        ix,
        vec![
            (STARKE_AUTHORITY, manager_account),
            (fake_vault_pubkey, vault_account),
            get_default_clock_account(),
        ]
    );
}
