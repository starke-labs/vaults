//! Guards the cluster-specific addresses selected by the `devnet` feature.
//! A default build must always carry the mainnet addresses.

use anchor_lang::prelude::*;

use crate::constants::STARKE_AUTHORITY;

#[cfg(not(feature = "devnet"))]
#[test]
fn default_build_uses_mainnet_addresses() {
    assert_eq!(
        crate::ID,
        pubkey!("56gFPCzaTGNJQcZrpfewDDgGYD8SR7G2RCrxy5z26jch")
    );
    assert_eq!(
        transfer_hook::ID,
        pubkey!("Gk7syLzEbk46Ez6Fr9pApPPhTJMDavKxiN9JHAtfhZCz")
    );
    assert_eq!(
        STARKE_AUTHORITY,
        pubkey!("STRK1me6eFLDYGKYqbn2oyHsaxiCHe8GDWQnnSGiScS")
    );
}

#[cfg(feature = "devnet")]
#[test]
fn devnet_build_uses_devnet_addresses() {
    assert_eq!(
        crate::ID,
        pubkey!("6UgbFY4Q8VQA67Zw4jsBjTzAVpFZWphVs32Efe4m6qho")
    );
    assert_eq!(
        transfer_hook::ID,
        pubkey!("8x1M8d2hJrD2cFhLN2SkDnaE3anSZ46EuexoewN9SgWn")
    );
    assert_eq!(
        STARKE_AUTHORITY,
        pubkey!("H2RYjGrqbq1EgcW3vrNF7UkHN76zem7VATKnvPNMuTH4")
    );
}
