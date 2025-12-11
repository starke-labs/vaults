pub mod add_manager;
pub mod add_token;
pub mod mint_management_fees;
pub mod close_vault;
pub mod create_vault;
pub mod deposit;
pub mod initialize_starke;
pub mod pause_starke;
pub mod remove_manager;
pub mod remove_token;
pub mod resume_starke;
pub mod set_vault_deposit_fee;
pub mod swap_on_jupiter;
pub mod swap_to_deposit_token_on_jupiter;
pub mod user_whitelist;
pub mod withdraw;
pub mod withdraw_in_deposit_token;

pub use add_manager::*;
pub use add_token::*;
pub use mint_management_fees::*;
pub use close_vault::*;
pub use create_vault::*;
pub use deposit::*;
pub use initialize_starke::*;
pub use pause_starke::*;
pub use remove_manager::*;
pub use remove_token::*;
pub use resume_starke::*;
pub use set_vault_deposit_fee::*;
pub use swap_on_jupiter::*;
pub use swap_to_deposit_token_on_jupiter::*;
pub use user_whitelist::*;
pub use withdraw::*;
pub use withdraw_in_deposit_token::*;

#[cfg(test)]
pub mod tests;
