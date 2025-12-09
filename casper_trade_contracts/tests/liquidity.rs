mod common;

use casper_trade_contracts::utils::{expand_to_18_decimals, expand_to_9_decimals};
use common::setup;
use odra::{casper_types::U256, host::HostRef, prelude::Addressable, uints::ToU512};

#[test]
fn liquidity_discrepency() {
    // Arrange: fresh env and deploy Factory with no fee_to configured
    let mut context = setup();

    let token0 = context.token0.address();
    let token1 = context.token1.address();
    let wcspr = context.wcspr.address();
    let pair1 = context.factory.create_pair(token0, wcspr);
    let pair2 = context.factory.create_pair(token1, wcspr);

    context
        .token0
        .approve(&context.router.address(), &U256::MAX);
    context
        .token1
        .approve(&context.router.address(), &U256::MAX);

    let amount_token_desired = expand_to_18_decimals(1000);
    let cspr_amount = expand_to_9_decimals(1000);

    let (amount_token, amount_cspr, liquidity) = context
        .router
        .with_tokens(cspr_amount.to_u512())
        .add_liquidity_cspr(
            token0,
            amount_token_desired,
            U256::zero(),
            U256::zero(),
            context.owner,
            u64::MAX,
        );

    // Get the pair for token0/WCSPR
    use casper_trade_contracts::pair::PairHostRef;
    let mut pair = PairHostRef::new(pair1, context.env.clone());

    // Approve the router to spend LP tokens
    pair.approve(&context.router.address(), &U256::MAX);

    // Remove all liquidity (minus MINIMUM_LIQUIDITY which is locked)
    let (removed_token, removed_cspr) = context.router.remove_liquidity_cspr(
        token0,
        liquidity, // Remove all our LP tokens
        U256::zero(),
        U256::zero(),
        context.owner,
        u64::MAX,
    );

    // Check the ratio is 1:1 (proportional to original deposits)
    // Original ratio: amount_token_desired (18 decimals) : cspr_amount (9 decimals)
    // Since token has 18 decimals and CSPR has 9 decimals, we need to normalize
    // Token/CSPR ratio should be preserved
    let original_ratio = amount_token * U256::exp10(9) / amount_cspr; // Normalize to same decimals
    let removed_ratio = removed_token * U256::exp10(9) / removed_cspr;

    // Ratios should be approximately equal (allow for small rounding due to MINIMUM_LIQUIDITY locked)
    // The difference should be very small (< 0.0001%)
    let ratio_diff = if removed_ratio > original_ratio {
        removed_ratio - original_ratio
    } else {
        original_ratio - removed_ratio
    };

    // Allow tolerance of 0.0001% (1 / 1_000_000)
    let tolerance = original_ratio / U256::from(1_000_000);
    assert!(
        ratio_diff <= tolerance,
        "Ratio mismatch after removing liquidity: original={}, removed={}, diff={}",
        original_ratio,
        removed_ratio,
        ratio_diff
    );

    // Check the RESERVES ratio (the "dust" left in the pair)
    let (reserve0_1, reserve1_1, _) = pair.get_reserves();
    let pair_token0_1 = pair.token0();
    let (reserve_token_1, reserve_cspr_1) = if pair_token0_1 == token0 {
        (reserve0_1, reserve1_1)
    } else {
        (reserve1_1, reserve0_1)
    };

    // Normalize reserves to same decimals and check ratio
    let reserve_ratio_1 = reserve_token_1 * U256::exp10(9) / reserve_cspr_1;
    // Case 1 values stored for final comparison

    // === Second test: smaller amounts (10 tokens, 10 CSPR) ===
    let amount_token_desired_2 = expand_to_18_decimals(10);
    let cspr_amount_2 = expand_to_9_decimals(10);

    let (amount_token_2, amount_cspr_2, liquidity_2) = context
        .router
        .with_tokens(cspr_amount_2.to_u512())
        .add_liquidity_cspr(
            token1,
            amount_token_desired_2,
            U256::zero(),
            U256::zero(),
            context.owner,
            u64::MAX,
        );

    // Get the pair for token1/WCSPR
    let mut pair2_instance = PairHostRef::new(pair2, context.env.clone());

    // Approve the router to spend LP tokens
    pair2_instance.approve(&context.router.address(), &U256::MAX);

    // Remove all liquidity
    let (removed_token_2, removed_cspr_2) = context.router.remove_liquidity_cspr(
        token1,
        liquidity_2,
        U256::zero(),
        U256::zero(),
        context.owner,
        u64::MAX,
    );

    // Check the ratio is preserved
    let original_ratio_2 = amount_token_2 * U256::exp10(9) / amount_cspr_2;
    let removed_ratio_2 = removed_token_2 * U256::exp10(9) / removed_cspr_2;

    let ratio_diff_2 = if removed_ratio_2 > original_ratio_2 {
        removed_ratio_2 - original_ratio_2
    } else {
        original_ratio_2 - removed_ratio_2
    };

    let tolerance_2 = original_ratio_2 / U256::from(1_000_000);
    assert!(
        ratio_diff_2 <= tolerance_2,
        "Ratio mismatch (10/10 case): original={}, removed={}, diff={}",
        original_ratio_2,
        removed_ratio_2,
        ratio_diff_2
    );

    // Check the RESERVES ratio for case 2 (the "dust" left in the pair)
    let (reserve0_2, reserve1_2, _) = pair2_instance.get_reserves();
    let pair_token0_2 = pair2_instance.token0();
    let (reserve_token_2, reserve_cspr_2) = if pair_token0_2 == token1 {
        (reserve0_2, reserve1_2)
    } else {
        (reserve1_2, reserve0_2)
    };

    // Normalize reserves to same decimals and check ratio
    let reserve_ratio_2 = reserve_token_2 * U256::exp10(9) / reserve_cspr_2;
    // Case 2 values stored for final comparison

    // The friend claims:
    // - Case 1 (1000/1000): reserve ratio should be 1:1 ✅
    // - Case 2 (10/10): reserve ratio is ~31:1 ⛔️
    // Let's verify! If both ratios are approximately 10^18 (1:1 when normalized),
    // then the ratios are preserved.
    // Assert that the reserve ratios are reasonably close (within 50% of each other)
    // If friend's claim is true, reserve_ratio_2 would be ~31x different
    let reserve_ratio_diff = if reserve_ratio_2 > reserve_ratio_1 {
        reserve_ratio_2 * U256::from(100) / reserve_ratio_1
    } else {
        reserve_ratio_1 * U256::from(100) / reserve_ratio_2
    };

    // If friend's claim is correct (31:1 vs 1:1), this would be ~3100
    // If our implementation is correct, this should be close to 100
    assert!(
        reserve_ratio_diff < U256::from(200), // Allow 2x tolerance
        "\nReserve ratios comparison:\n  Case 1 (1000/1000): token={}, cspr={}, ratio={}\n  Case 2 (10/10): token={}, cspr={}, ratio={}\n  Diff%={} (should be ~100 if equal, ~3100 if friend is right)",
        reserve_token_1, reserve_cspr_1, reserve_ratio_1,
        reserve_token_2, reserve_cspr_2, reserve_ratio_2,
        reserve_ratio_diff
    );

    // ========================================================================
    // DETAILED OUTPUT FOR DEBUGGING FRIEND'S CLAIM
    // ========================================================================
    //
    // IMPORTANT: The RAW reserve ratio after full withdrawal is NOT expected to be 10^9!
    //
    // Here's why: The MINIMUM_LIQUIDITY (1000 LP tokens) locked forever represents
    // a GEOMETRIC mean of the two token amounts, not an arithmetic ratio.
    //
    // For initial liquidity with token (18 dec) and CSPR (9 dec):
    //   LP_minted = sqrt(token_amount * cspr_amount) - MINIMUM_LIQUIDITY
    //
    // The locked reserves correspond to MINIMUM_LIQUIDITY / total_supply of each token.
    // Since total_supply = sqrt(token * cspr), the locked reserves are:
    //   locked_token = token * MINIMUM_LIQUIDITY / sqrt(token * cspr)
    //   locked_cspr = cspr * MINIMUM_LIQUIDITY / sqrt(token * cspr)
    //
    // This means the RAW ratio of locked reserves = token / cspr (original ratio!)
    // But when tokens have different decimals, this looks different.
    //
    // What matters is that BOTH cases have the SAME normalized ratio - which we
    // already verified above with `reserve_ratio_diff < 200`.

    // The key insight: The friend observed raw reserve ratio and thought it was 31:1,
    // but that's because they didn't account for the decimal difference.
    //
    // Let's show what the raw reserves actually are:
    let raw_ratio_1 = reserve_token_1 / reserve_cspr_1;
    let raw_ratio_2 = reserve_token_2 / reserve_cspr_2;

    // Both raw ratios should be the SAME because they represent the same price ratio
    // (just expressed in raw token units, not normalized value)
    let raw_ratio_comparison = if raw_ratio_2 > raw_ratio_1 {
        raw_ratio_2 * U256::from(100) / raw_ratio_1
    } else {
        raw_ratio_1 * U256::from(100) / raw_ratio_2
    };

    // Both raw ratios should be essentially equal (within 1% tolerance)
    assert!(
        raw_ratio_comparison >= U256::from(99) && raw_ratio_comparison <= U256::from(101),
        "Raw reserve ratios should be equal between cases!\n  Case 1 raw: {}\n  Case 2 raw: {}\n  Comparison: {}%",
        raw_ratio_1, raw_ratio_2, raw_ratio_comparison
    );
}

/// This test outputs all the key values for manual inspection
/// Run with: cargo test -p casper_trade_contracts --test liquidity -- debug_reserves_output --nocapture
#[test]
fn debug_reserves_output() {
    use casper_trade_contracts::pair::PairHostRef;
    use casper_trade_contracts::pair::MINIMUM_LIQUIDITY;

    let mut context = setup();

    let token0 = context.token0.address();
    let token1 = context.token1.address();
    let wcspr = context.wcspr.address();
    let pair1 = context.factory.create_pair(token0, wcspr);
    let pair2 = context.factory.create_pair(token1, wcspr);

    context
        .token0
        .approve(&context.router.address(), &U256::MAX);
    context
        .token1
        .approve(&context.router.address(), &U256::MAX);

    // Case 1: 1000 tokens, 1000 CSPR
    let token_amount_1 = expand_to_18_decimals(1000);
    let cspr_amount_1 = expand_to_9_decimals(1000);

    let (actual_token_1, actual_cspr_1, lp_1) = context
        .router
        .with_tokens(cspr_amount_1.to_u512())
        .add_liquidity_cspr(
            token0,
            token_amount_1,
            U256::zero(),
            U256::zero(),
            context.owner,
            u64::MAX,
        );

    let mut pair1_instance = PairHostRef::new(pair1, context.env.clone());
    let (r0_before_1, r1_before_1, _) = pair1_instance.get_reserves();

    pair1_instance.approve(&context.router.address(), &U256::MAX);
    let (withdrawn_token_1, withdrawn_cspr_1) = context.router.remove_liquidity_cspr(
        token0,
        lp_1,
        U256::zero(),
        U256::zero(),
        context.owner,
        u64::MAX,
    );

    let (r0_after_1, r1_after_1, _) = pair1_instance.get_reserves();

    // Case 2: 10 tokens, 10 CSPR
    let token_amount_2 = expand_to_18_decimals(10);
    let cspr_amount_2 = expand_to_9_decimals(10);

    let (actual_token_2, actual_cspr_2, lp_2) = context
        .router
        .with_tokens(cspr_amount_2.to_u512())
        .add_liquidity_cspr(
            token1,
            token_amount_2,
            U256::zero(),
            U256::zero(),
            context.owner,
            u64::MAX,
        );

    let mut pair2_instance = PairHostRef::new(pair2, context.env.clone());
    let (r0_before_2, r1_before_2, _) = pair2_instance.get_reserves();

    pair2_instance.approve(&context.router.address(), &U256::MAX);
    let (withdrawn_token_2, withdrawn_cspr_2) = context.router.remove_liquidity_cspr(
        token1,
        lp_2,
        U256::zero(),
        U256::zero(),
        context.owner,
        u64::MAX,
    );

    let (r0_after_2, r1_after_2, _) = pair2_instance.get_reserves();

    // Output for debugging
    let pair1_token0 = pair1_instance.token0();
    let pair2_token0 = pair2_instance.token0();

    // Panic with all the values so they're visible in test output
    panic!(
        "\n\n=== LIQUIDITY TEST DEBUG OUTPUT ===\n\n\
        MINIMUM_LIQUIDITY = {}\n\n\
        === CASE 1: 1000 Token (18 dec) / 1000 CSPR (9 dec) ===\n\
        Deposited:   token={}, cspr={}\n\
        Actual used: token={}, cspr={}\n\
        LP received: {}\n\
        Reserves before withdrawal: r0={}, r1={}\n\
        Withdrawn:   token={}, cspr={}\n\
        Reserves after withdrawal:  r0={}, r1={}\n\
        Pair token0 is: {:?}\n\
        Token0 == deposited token? {}\n\n\
        === CASE 2: 10 Token (18 dec) / 10 CSPR (9 dec) ===\n\
        Deposited:   token={}, cspr={}\n\
        Actual used: token={}, cspr={}\n\
        LP received: {}\n\
        Reserves before withdrawal: r0={}, r1={}\n\
        Withdrawn:   token={}, cspr={}\n\
        Reserves after withdrawal:  r0={}, r1={}\n\
        Pair token0 is: {:?}\n\
        Token1 == deposited token? {}\n\n\
        === RATIO ANALYSIS ===\n\
        Case 1 after-withdrawal reserves ratio (normalized): {}\n\
        Case 2 after-withdrawal reserves ratio (normalized): {}\n\
        (Both should be ~1e18 for 1:1 price ratio)\n\n\
        =================================\n",
        MINIMUM_LIQUIDITY,
        token_amount_1,
        cspr_amount_1,
        actual_token_1,
        actual_cspr_1,
        lp_1,
        r0_before_1,
        r1_before_1,
        withdrawn_token_1,
        withdrawn_cspr_1,
        r0_after_1,
        r1_after_1,
        pair1_token0,
        pair1_token0 == token0,
        token_amount_2,
        cspr_amount_2,
        actual_token_2,
        actual_cspr_2,
        lp_2,
        r0_before_2,
        r1_before_2,
        withdrawn_token_2,
        withdrawn_cspr_2,
        r0_after_2,
        r1_after_2,
        pair2_token0,
        pair2_token0 == token1,
        // Case 1: token0 IS the deposited token, so r0=token, r1=cspr
        // ratio = token * 10^9 / cspr = r0 * 10^9 / r1
        if pair1_token0 == token0 {
            r0_after_1 * U256::exp10(9) / r1_after_1
        } else {
            r1_after_1 * U256::exp10(9) / r0_after_1
        },
        // Case 2: need to check token ordering - wcspr comes before some tokens alphabetically
        if pair2_token0 == token1 {
            r0_after_2 * U256::exp10(9) / r1_after_2
        } else {
            // token1 is in r1, wcspr is in r0
            r1_after_2 * U256::exp10(9) / r0_after_2
        }
    );
}
