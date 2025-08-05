pub mod add_manager;
pub mod add_token;
pub mod create_vault;
pub mod deposit;
pub mod initialize_starke;
pub mod pause_starke;
pub mod remove_manager;
pub mod remove_token;
pub mod resume_starke;
pub mod swap_on_jupiter;
pub mod update_vault_fees;
pub mod user_whitelist;
pub mod withdraw;

pub use add_manager::*;
pub use add_token::*;
pub use create_vault::*;
pub use deposit::*;
pub use initialize_starke::*;
pub use pause_starke::*;
pub use remove_manager::*;
pub use remove_token::*;
pub use resume_starke::*;
pub use swap_on_jupiter::*;
pub use update_vault_fees::*;
pub use user_whitelist::*;
pub use withdraw::*;

#[cfg(test)]
pub mod tests;
