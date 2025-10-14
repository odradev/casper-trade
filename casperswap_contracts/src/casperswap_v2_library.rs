use alloc::rc::Rc;
use odra::{casper_types::U256, prelude::*, ContractEnv, ContractRef};

use crate::casperswap_v2_pair::CasperswapV2PairContractRef;

/// CasperswapV2Library - Library functions for router calculations
/// Based on UniswapV2Library

pub mod errors {
    use odra::prelude::*;

    #[odra::odra_error]
    pub enum CasperswapV2LibraryError {
        IdenticalAddresses = 1,
        ZeroAddress = 2,
        InsufficientAmount = 3,
        InsufficientLiquidity = 4,
        InsufficientInputAmount = 5,
        InsufficientOutputAmount = 6,
        InvalidPath = 7,
    }
}

use errors::CasperswapV2LibraryError;

/// Returns sorted token addresses, used to handle return values from pairs sorted in this order
pub fn sort_tokens(env: &ContractEnv, token_a: Address, token_b: Address) -> (Address, Address) {
    if token_a == token_b {
        env.revert(CasperswapV2LibraryError::IdenticalAddresses);
    }
    let (token0, token1) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };
    // Check if token0 is zero address (using a utility function from utils module)
    if token0 == crate::utils::zero_address() {
        env.revert(CasperswapV2LibraryError::ZeroAddress);
    }
    (token0, token1)
}

/// Calculates the CREATE2 address for a pair without making any external calls
/// Note: In Casper, we'll need to use the factory's get_pair method instead
pub fn pair_for(_factory: Address, _token_a: Address, _token_b: Address) -> Address {
    // TODO: Implement pair address calculation
    // For now, we'll need to call the factory's get_pair method
    // This is a placeholder that will be updated when we implement the factory's get_pair
    unimplemented!("pair_for needs factory.get_pair() implementation")
}

/// Fetches and sorts the reserves for a pair
pub fn get_reserves(
    env: &ContractEnv,
    factory: Address,
    token_a: Address,
    token_b: Address,
) -> (U256, U256, Address) {
    let (token0, _token1) = sort_tokens(env, token_a, token_b);
    let pair_address = pair_for(factory, token_a, token_b);
    let pair = CasperswapV2PairContractRef::new(Rc::new(env.clone()), pair_address);
    let (reserve0, reserve1, _) = pair.get_reserves();
    let (reserve_a, reserve_b) = if token_a == token0 {
        (reserve0, reserve1)
    } else {
        (reserve1, reserve0)
    };
    (reserve_a, reserve_b, pair_address)
}

/// Given some amount of an asset and pair reserves, returns an equivalent amount of the other asset
pub fn quote(env: &ContractEnv, amount_a: U256, reserve_a: U256, reserve_b: U256) -> U256 {
    if amount_a.is_zero() {
        env.revert(CasperswapV2LibraryError::InsufficientAmount);
    }
    if reserve_a.is_zero() || reserve_b.is_zero() {
        env.revert(CasperswapV2LibraryError::InsufficientLiquidity);
    }
    amount_a * reserve_b / reserve_a
}

/// Given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset
pub fn get_amount_out(env: &ContractEnv, amount_in: U256, reserve_in: U256, reserve_out: U256) -> U256 {
    if amount_in.is_zero() {
        env.revert(CasperswapV2LibraryError::InsufficientInputAmount);
    }
    if reserve_in.is_zero() || reserve_out.is_zero() {
        env.revert(CasperswapV2LibraryError::InsufficientLiquidity);
    }
    let amount_in_with_fee = amount_in * U256::from(997);
    let numerator = amount_in_with_fee * reserve_out;
    let denominator = reserve_in * U256::from(1000) + amount_in_with_fee;
    numerator / denominator
}

/// Given an output amount of an asset and pair reserves, returns a required input amount of the other asset
pub fn get_amount_in(env: &ContractEnv, amount_out: U256, reserve_in: U256, reserve_out: U256) -> U256 {
    if amount_out.is_zero() {
        env.revert(CasperswapV2LibraryError::InsufficientOutputAmount);
    }
    if reserve_in.is_zero() || reserve_out.is_zero() {
        env.revert(CasperswapV2LibraryError::InsufficientLiquidity);
    }
    let numerator = reserve_in * amount_out * U256::from(1000);
    let denominator = (reserve_out - amount_out) * U256::from(997);
    (numerator / denominator) + U256::from(1)
}

/// Performs chained getAmountOut calculations on any number of pairs
pub fn get_amounts_out(env: &ContractEnv, factory: Address, amount_in: U256, path: Vec<Address>) -> Vec<U256> {
    if path.len() < 2 {
        env.revert(CasperswapV2LibraryError::InvalidPath);
    }
    let mut amounts = vec![U256::zero(); path.len()];
    amounts[0] = amount_in;
    for i in 0..path.len() - 1 {
        let (reserve_in, reserve_out, _) = get_reserves(env, factory, path[i], path[i + 1]);
        amounts[i + 1] = get_amount_out(env, amounts[i], reserve_in, reserve_out);
    }
    amounts
}

/// Performs chained getAmountIn calculations on any number of pairs
pub fn get_amounts_in(env: &ContractEnv, factory: Address, amount_out: U256, path: Vec<Address>) -> Vec<U256> {
    if path.len() < 2 {
        env.revert(CasperswapV2LibraryError::InvalidPath);
    }
    let mut amounts = vec![U256::zero(); path.len()];
    let len = amounts.len();
    amounts[len - 1] = amount_out;
    for i in (1..path.len()).rev() {
        let (reserve_in, reserve_out, _) = get_reserves(env, factory, path[i - 1], path[i]);
        let current_amount = amounts[i];
        amounts[i - 1] = get_amount_in(env, current_amount, reserve_in, reserve_out);
    }
    amounts
}

// Tests will be added when the full implementation is complete
// #[cfg(test)]
// mod tests {}

