mod common;

use casper_trade_contracts::pair::errors::PairError::InsufficientLiquidityMinted;
use common::setup;
use odra::casper_types::U256;

// (HAL-07) MISSING ERROR HANDLING IN INITIAL MINT
#[test]
fn test_missing_error_handling_in_initial_mint() {
    let mut context = setup();

    // Before the fix, this would yield "ExecError: Interpreter error: trap: Code(Unreachable)"
    // context.add_liquidity(U256::from(5), U256::from(10));

    // After the fix, it reverts with PairError::InsufficientLiquidityMinted
    let error = context
        .add_liquidity(U256::from(5), U256::from(10))
        .unwrap_err();
    assert_eq!(error, InsufficientLiquidityMinted.into())
}
