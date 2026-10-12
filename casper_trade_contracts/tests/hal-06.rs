mod common;

use common::setup;
use odra::prelude::*;

// (HAL-06) UNDISTINGUISHABLE LP TOKEN METADATA
// ACROSS ALL PAIRS
#[test]
fn test_undistinguishable_lp_token_metadata() {
    let context = setup();

    // Before the adjustment, pair symbol and name would be hardcoded
    // even when exposed.
    let pair_symbol = context.pair.symbol();
    let pair_name = context.pair.name();
    // assert_eq!(pair_symbol, "LP".to_string());
    // assert_eq!(pair_name, "CasperTradeV2Pair".to_string());

    // The adjustment generates unique names and symbols for the pair,
    // built from its tokens in the order the factory sorts them:
    let (first, second) = if context.token0.address() < context.token1.address() {
        (&context.token0, &context.token1)
    } else {
        (&context.token1, &context.token0)
    };
    assert_eq!(
        pair_name,
        format!("CSPR.trade: {} - {}", first.name(), second.name())
    );
    assert_eq!(
        pair_symbol,
        format!("{}-{} LP", first.symbol(), second.symbol())
    );
}
