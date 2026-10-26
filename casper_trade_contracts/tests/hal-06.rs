mod common;

use common::setup;

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

    // The adjustment generates unique names and symbols for the pair:
    assert_eq!(
        pair_name,
        "CSPR.trade: Sample Token A - Sample Token B".to_string()
    );
    assert_eq!(pair_symbol, "STA-STB LP");
}
