mod common;

use common::setup;

// (HAL-04) MISSING DECIMAL-ADJUSTMENT METADATA
// FOR TWAP CONSUMERS
#[test]
fn test_missing_decimal_adjustment_metadata() {
    let context = setup();
    let token0_decimals = context.token0.decimals();
    let token1_decimals = context.token1.decimals();

    // It is now possible to query the pair for token decimals.
    let tokens_decimals = context.pair.get_tokens_decimals();
    assert_eq!(token0_decimals, tokens_decimals.0);
    assert_eq!(token1_decimals, tokens_decimals.1);
}
