mod common;

use casper_trade_contracts::router::errors::LibraryError::{IdenticalAddresses, ZeroAddress};
use casper_trade_contracts::utils::zero_address;
use common::setup;
use odra::prelude::*;

// (HAL-03) FACTORY ALLOWS CREATING PAIRS WITH ZERO
// OR IDENTICAL TOKEN ADDRESSES
#[test]
fn test_factory_allows_creating_pairs_with_identical_addresses() {
    let mut context = setup();

    let token_1 = context.token1.address();
    let zero_address = zero_address();

    // Before fix, it was possible to create a pair with zero address
    // let zero_pair = context.factory.try_create_pair(token_1, zero_address);
    // assert!(zero_pair.is_ok());

    // Or pair with two identical tokens:
    // let identical_pair = context.factory.try_create_pair(token_1, token_1);
    // assert!(identical_pair.is_ok());

    // After the fix, both operations revert:
    let zero_pair = context.factory.try_create_pair(token_1, zero_address);
    assert_eq!(zero_pair.unwrap_err(), ZeroAddress.into());

    // Or pair with two identical tokens:
    let identical_pair = context.factory.try_create_pair(token_1, token_1);
    assert_eq!(identical_pair.unwrap_err(), IdenticalAddresses.into());
}
