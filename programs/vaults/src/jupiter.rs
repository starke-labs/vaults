use anchor_lang::{prelude::*, Id};

#[derive(Clone)]
pub struct Jupiter;

impl Id for Jupiter {
    fn id() -> Pubkey {
        jupiter::id()
    }
}
