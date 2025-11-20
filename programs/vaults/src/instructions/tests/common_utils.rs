use solana_sdk::{account::Account, pubkey::Pubkey, sysvar::clock};

/// Warning: Implicit unsafe usage! Only for use in testing.
/// Returns a default Sysvar `Clock` account tuple consisting of the clock program's `Pubkey` and a corresponding `Account`.
///
/// # Returns
///
/// A tuple `(Pubkey, Account)` where:
/// - `Pubkey` is the ID of the Sysvar clock account.
/// - `Account` is an initialized clock account with default `Clock` data, owned by the Sysvar program.
///
/// # Notes
///
/// - The clock data is obtained using `Clock::default()` and converted to bytes using unsafe memory casting.
/// - This function is intended for testing and simulation purposes only.
///
/// # Safety
///
/// Uses `unsafe` code to convert a `Clock` struct into a byte slice.
pub(super) fn get_default_clock_account() -> (Pubkey, Account) {
    let clock_meta = clock::Clock::default();
    // TODO: Make sure `Clock` has a `repr(C)` layout and contains no pointers to ensure safe casting.
    let clock_bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(
            (&clock_meta as *const clock::Clock) as *const u8,
            size_of::<clock::Clock>(),
        )
    };
    (
        clock::id(),
        Account {
            lamports: 1,
            data: clock_bytes.to_vec(),
            owner: solana_sdk::sysvar::id(),
            executable: false,
            rent_epoch: 0,
        },
    )
}

/// Returns a default `Account` owned by the given program ID, typically used for simulating signers or executable program accounts.
///
/// # Parameters
///
/// - `program_id`: The `Pubkey` of the program that will own the account.
///
/// # Returns
///
/// An `Account` with the following properties:
/// - `lamports`: 1_000_000_000
/// - `data`: Empty vector
/// - `owner`: The provided `program_id`
/// - `executable`: `true`
/// - `rent_epoch`: `0`
///
/// # Use Case
///
/// Useful for simulating authority or executable accounts in unit tests or program test harnesses.
pub(super) fn new_default_account(program_id: Pubkey) -> Account {
    Account {
        lamports: 1_000_000_000,
        data: vec![],
        owner: program_id,
        executable: true,
        rent_epoch: 0,
    }
}

#[allow(dead_code)]
pub(super) fn new_default_account_with_space(program_id: Pubkey, space: usize) -> Account {
    Account {
        lamports: 1_000_000_000,
        data: vec![0; space],
        owner: program_id,
        executable: false,
        rent_epoch: 0,
    }
}

/// Defines a default instruction constructor function with a specific discriminator, serialized arguments, and account metadata.
///
/// # Macro Parameters
///
/// - `$name`: The name of the generated function.
/// - `hash = $hash`: A string literal used to generate the instruction discriminator (first 8 bytes of the SHA256 hash).
/// - `args = [ name: type, ... ]`: Regular arguments passed to the function but not serialized into the instruction data.
/// - `ser_args = [ name: type => serializer, ... ]`: Arguments that are serialized into the instruction data using the given expressions.
/// - `accounts = [ (account_expr, is_signer), ... ]`: Account metadata to include in the instruction.
///
/// # Generated Function Signature
///
/// ```rust
/// pub fn $name(
///     program_id: Pubkey,
///     $( $arg_name : $arg_ty, )*
///     $( $ser_arg_name : $ser_arg_ty, )*
/// ) -> Result<Instruction>
/// ```
///
/// # Returns
///
/// Returns a `Result<Instruction>` representing the constructed instruction, including:
/// - The 8-byte discriminator prefix based on the given hash.
/// - Serialized argument data.
/// - A vector of `AccountMeta` entries as defined.
///
/// # Example
///
/// ```rust
/// def_default_instruction!(
///     my_instruction,
///     hash = "my_instruction",
///     args = [user: Pubkey],
///     ser_args = [amount: u64 => amount.to_le_bytes()],
///     accounts = [(user, true), (system_program::ID, false)],
/// );
///
/// // Usage:
/// let ix = my_instruction(program_id, user_pubkey, 100)?;
/// ```
///
/// # Notes
///
/// - The `hash` is used to generate the instruction discriminator prefix using SHA-256.
/// - Serialization logic for each `ser_arg` must be explicitly provided with `.to_le_bytes()` or other appropriate serializers.
/// - Account expressions are evaluated in the context of the function call.
#[macro_export]
macro_rules! def_default_instruction {
    (
        $name:ident,
        pid = $pid:expr,
        hash = $hash:expr,
        args = [ $( $arg_name:ident : $arg_ty:ty ),* $(,)? ],
        ser_args = [ $( $ser_arg_name:ident : $ser_arg_ty:ty => $serializer:expr ),* $(,)? ],
        accounts = [ $( ($account:expr, $is_signer:expr) ),* $(,)? ]
        $(, accounts_readonly = [ $( ($account_ro:expr, $is_signer_ro:expr) ),* $(,)? ] )? $(,)?
    ) => {
        #[allow(clippy::too_many_arguments)]
        pub fn $name(
            $( $arg_name : $arg_ty, )*
            $( $ser_arg_name : $ser_arg_ty, )*
        ) -> Result<solana_program::instruction::Instruction> {
            let instruction_data = {
                let mut data = vec![];

                {
                    use anchor_lang::solana_program::hash::hash;
                    let full_hash = hash($hash.as_bytes()).to_bytes();
                    data.extend_from_slice(&full_hash[..8]);
                }

                $(
                    data.extend_from_slice(&$serializer);
                )*

                data
            };

            let accounts = vec![
                $(
                    solana_program::instruction::AccountMeta::new($account, $is_signer),
                )*
                $(
                    $(
                        solana_program::instruction::AccountMeta::new_readonly($account_ro, $is_signer_ro),
                    )*
                )?
            ];


            Ok(solana_program::instruction::Instruction {
                program_id: $pid,
                accounts,
                data: instruction_data,
            })
        }
    };
}

/// Executes a Solana instruction using the `mollusk_svm` simulation environment.
///
/// # Parameters
/// - `program_id`: The `Pubkey` of the program to invoke.
/// - `ix`: The `Instruction` to execute.
/// - `accounts`: A vector of `AccountInfo`-like objects representing the account context for the instruction.
///
/// # Details
///
/// This macro uses the `mollusk_svm::Mollusk` simulation runtime to process a given instruction
/// against a specified program, loading the program binary from the relative path
/// `"../../target/deploy/vaults"`.
///
/// # Examples
///
/// ```rust
/// let result = run_ix!(program_id, instruction, accounts);
/// assert!(result.raw_result.is_ok());
/// ```
///
/// # Notes
///
/// Ensure that the program has been built and deployed locally so that the binary exists
/// at the expected path when using this macro.
#[macro_export]
macro_rules! run_ix {
    ($program_id:expr, $ix:expr, $accounts: expr) => {{
        let mut mollusk = mollusk_svm::Mollusk::new(&$program_id, "../../target/deploy/vaults");
        mollusk.config.verbose = true;

        mollusk.process_instruction(&$ix, &$accounts)
    }};
}

/// Executes a Solana instruction using `run_ix!` and expects it to succeed.
///
/// # Example
/// ```
/// let result = run_ix_expect!(
///     program_id,
///     instruction,
///     vec![
///         (some_pubkey, some_account),
///         ...
///     ]
/// );
/// ```
///
/// If the instruction fails, this macro will panic with the result's error.
#[macro_export]
macro_rules! run_ix_expect {
    ($program_id:expr, $ix:expr, $accounts:expr $(,)?) => {{
        let result = run_ix!($program_id, $ix, $accounts);
        assert!(
            result.raw_result.is_ok(),
            "Instruction failed unexpectedly: {:#?}",
            result
        );
        result
    }};
}

/// Executes a Solana instruction using `run_ix!` and expects it to fail.
///
/// # Example
/// ```
/// run_ix_expect_err!(
///     program_id,
///     bad_instruction,
///     vec![
///         (some_pubkey, some_account),
///         ...
///     ]
/// );
/// ```
///
/// If the instruction unexpectedly succeeds, this macro will panic.
#[macro_export]
macro_rules! run_ix_expect_err {
    ($program_id:expr, $ix:expr, $accounts:expr $(,)?) => {{
        let result = run_ix!($program_id, $ix, $accounts);
        assert!(
            result.raw_result.is_err(),
            "Instruction unexpectedly succeeded when failure was expected. {:#?}",
            result
        );
        result
    }};
}

/// Creates a whitelisted `Account` with serialized state data and customizable parameters.
///
/// # Parameters
///
/// - `state`: The state object to serialize into the account's data buffer. Must implement `borsh::BorshSerialize`.
/// - `program_id`: The program ID that will own the account.
///
/// # Optional Parameters
///
/// - `lamports` *(optional)*: The number of lamports to assign to the account. Defaults to `1_000_000_000`.
/// - `executable` *(optional)*: Whether the account is executable. Defaults to `false`.
/// - `rent_epoch` *(optional)*: The epoch at which the account will next owe rent. Defaults to `0`.
///
/// # Returns
///
/// An `Account` struct initialized with the provided or default values and serialized state data.
///
/// # Panics
///
/// Panics if state serialization fails.
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// let account = serialize_into_account!(state, program_id);
/// ```
///
/// With custom lamports:
/// ```rust
/// let account = serialize_into_account!(state, program_id, 2_000_000_000);
/// ```
///
/// With custom lamports and executable flag:
/// ```rust
/// let account = serialize_into_account!(state, program_id, 2_000_000_000, true);
/// ```
///
/// With all custom parameters:
/// ```rust
/// let account = serialize_into_account!(state, program_id, 2_000_000_000, true, 42);
/// ```
#[macro_export]
macro_rules! serialize_into_account {
    // Default: only state and program_id
    ($state:expr, $program_id:expr) => {
        serialize_into_account!($state, $program_id, 1_000_000_000, false, 0)
    };

    // With lamports
    ($state:expr, $program_id:expr, $lamports:expr) => {
        serialize_into_account!($state, $program_id, $lamports, false, 0)
    };

    // With lamports and executable
    ($state:expr, $program_id:expr, $lamports:expr, $executable:expr) => {
        serialize_into_account!($state, $program_id, $lamports, $executable, 0)
    };

    // Full form
    ($state:expr, $program_id:expr, $lamports:expr, $executable:expr, $rent_epoch:expr) => {{
        let mut data = vec![0u8; 1024];
        $state.try_serialize(&mut &mut data[..]).expect(&format!(
            "Failed to serialize initial {} state",
            stringify!($state),
        ));

        Account {
            lamports: $lamports,
            data,
            owner: $program_id,
            executable: $executable,
            rent_epoch: $rent_epoch,
        }
    }};
}

/* Deprecated
macro_rules! derive_mock_identity_for {
    ($struct:ident, $pid:expr) => {{
        let (pda, bump) = Pubkey::find_program_address(&[&$struct::SEED], &$pid);
        (Pubkey::new_unique(), pda, bump)
    }};
}
*/

/*   Reserved for later usage.
#[macro_export]
macro_rules! mint_account {
    ($mint_pubkey:expr, $supply:expr) => {{
        let mint = Mint {
            mint_authority: COption::None,
            supply: $supply,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        };

        let mut data = vec![0u8; Mint::LEN];
        mint.pack_into_slice(&mut data);

        (
            $mint_pubkey,
            Account {
                lamports: 1_000_000_000,
                data,
                owner: anchor_spl::token::ID,
                executable: false,
                rent_epoch: 0,
            },
        )
    }};
}

#[macro_export]
macro_rules! token_account {
    ($owner:expr, $mint:expr, $amount:expr) => {{
        use solana_sdk::account::Account;
        use spl_token::state::Account as TokenAccount;

        let token_account = TokenAccount {
            mint: $mint,
            owner: $owner,
            amount: $amount,
            delegate: COption::None,
            state: spl_token::state::AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        };

        let mut data = vec![0u8; TokenAccount::LEN];
        token_account.pack_into_slice(&mut data);

        (
            Pubkey::new_unique(),
            Account {
                lamports: 1_000_000_000,
                data,
                owner: spl_token::id(),
                executable: false,
                rent_epoch: 0,
            },
        )
    }};
}
*/

pub(super) use def_default_instruction;
pub(super) use run_ix;
pub(super) use run_ix_expect;
pub(super) use run_ix_expect_err;
pub(super) use serialize_into_account;
//pub(super) use derive_mock_identity_for;
//pub(super) use token_account;
//pub(super) use mint_account;
