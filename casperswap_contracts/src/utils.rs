use core::str::FromStr;

use odra::{casper_types::U256, prelude::Address};

pub fn expand_to_18_decimals(amount: u64) -> U256 {
    let amount = U256::from(amount);
    amount.saturating_mul(U256::from(10).pow(U256::from(18)));
    amount
}
