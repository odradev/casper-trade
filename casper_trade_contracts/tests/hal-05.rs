mod common;

// use casper_trade_contracts::pair::errors::PairError::Forbidden;
// use casper_trade_contracts::pair::{Pair, PairInitArgs};
// use common::setup;
// use odra::host::Deployer;
// use odra::prelude::Addressable;

// (HAL-05) MISLEADING AUTHORIZATION ERROR ON
// PAIR.INITIALIZE
#[test]
#[ignore]
fn test_missing_decimal_adjustment_metadata() {
    // This test is invalid, as we merged init and initialize.
    //
    // let context = setup();
    // let mut pair = Pair::deploy(
    //     &context.env,
    //     PairInitArgs {
    //         factory: context.factory.address(),
    //     },
    // );
    //
    // // Before the adjustment, initializing a pair by someone else than a factory would
    // // result with Overflow error.
    // let initialize_result = pair
    //     .try_initialize(context.token0.address(), context.token1.address())
    //     .unwrap_err();
    // // assert_eq!(initialize_result, Overflow.into());
    //
    // // Now it reverts with Forbidden
    // assert_eq!(initialize_result, Forbidden.into());
}
