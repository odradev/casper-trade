use core::str::FromStr;

use odra::{casper_types::U256, prelude::Address};

pub fn expand_to_18_decimals(amount: u64) -> U256 {
    let amount = U256::from(amount);
    amount.saturating_mul(U256::from(10).pow(U256::from(18)))
}

pub fn expand_to_9_decimals(amount: u64) -> U256 {
    let amount = U256::from(amount);
    amount.saturating_mul(U256::from(10).pow(U256::from(9)))
}

pub fn zero_address() -> Address {
    Address::from_str("hash-0000000000000000000000000000000000000000000000000000000000000000")
        .unwrap()
}

/// Encode prices in UQ112x112 format
/// Returns [price0, price1] where:
/// - price0 = reserve1 * 2^112 / reserve0 (price of token0 in terms of token1)
/// - price1 = reserve0 * 2^112 / reserve1 (price of token1 in terms of token0)
pub fn encode_price(reserve0: U256, reserve1: U256) -> (U256, U256) {
    let q112 = U256::from(2u128).pow(U256::from(112));
    let price0 = (reserve1 * q112) / reserve0;
    let price1 = (reserve0 * q112) / reserve1;
    (price0, price1)
}
