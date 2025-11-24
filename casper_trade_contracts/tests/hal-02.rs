mod common;

use casper_trade_contracts::utils::expand_to_18_decimals;
use common::{add_liquidity, setup};
use odra::casper_types::U256;
use odra::prelude::*;

// (HAL-02) REVERSED TOKEN ORDER CAUSES RESERVE
// MISMATCH IN ADD_LIQUIDITY
#[test]
fn test_add_liquidity_input_order_bug_poc() {
    // Seed pair with asymmetric reserves reserve0=1, reserve1=4 (token0:token1)
    let mut context = setup();

    add_liquidity(
        &mut context,
        expand_to_18_decimals(1),
        expand_to_18_decimals(4),
    );

    // Approvals
    context
        .token0
        .approve(&context.router.address(), &U256::MAX);
    context
        .token1
        .approve(&context.router.address(), &U256::MAX);

    let t0 = context.pair.token0();
    let t1 = context.pair.token1();

    // Correct order call (use asymmetric desired amounts)
    let desired_a1 = expand_to_18_decimals(2);
    let desired_b1 = expand_to_18_decimals(3);
    let (a1, b1, _l1) = context.router.add_liquidity(
        t0,
        t1,
        desired_a1,
        desired_b1,
        U256::from(0),
        U256::from(0),
        context.owner,
        u64::MAX,
    );

    // Reversed order call (inputs swapped) with swapped desireds
    let (a2, b2, _l2) = context.router.add_liquidity(
        t1,
        t0,
        desired_b1,
        desired_a1,
        U256::from(0),
        U256::from(0),
        context.owner,
        u64::MAX,
    );

    // Before fix:
    // And also different from swapping the correct-order outputs
    // assert!( a2 != b1 || b2 != a1,
    //     "reversed-order amounts unexpectedly equal swapped correct-order amounts"
    // );

    // After fix:
    // Order doesn't affect the values:
    assert!(
        a2 == b1 && b2 == a1,
        "reversed-order amounts don't equal swapped correct-order amounts"
    );
}
