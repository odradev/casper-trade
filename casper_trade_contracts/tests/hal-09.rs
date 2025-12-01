mod common;

use casper_trade_contracts::pair::events::{
    CSPRRefunded, FactoryInitialized, FeeToUpdated, PairInitialized, ProtocolFeeMinted, SkimExcess,
};
use casper_trade_contracts::pair::MINIMUM_LIQUIDITY;
use casper_trade_contracts::utils::{expand_to_18_decimals, expand_to_9_decimals};
use common::setup;
use odra::casper_types::U256;
use odra::host::HostRef;
use odra::prelude::Addressable;
use odra::uints::ToU512;

// (HAL-09) MISSING EVENTS ON CRITICAL STATE CHANGES
// REDUCES AUDITABILITY AND MONITORING
#[test]
fn test_missing_events_on_critical_state_changes() {
    let mut context = setup();

    // FeeToUpdated
    let alice = context.env.get_account(1);
    context.factory.set_fee_to(Some(alice));
    assert!(context.env.emitted_event(
        &context.factory.address(),
        FeeToUpdated {
            old: None,
            new: Some(alice)
        }
    ));

    // FactoryInitialized
    assert!(context.env.emitted_event(
        &context.factory.address(),
        FactoryInitialized {
            fee_to: None,
            pair_factory: context.pair_factory.address(),
        }
    ));

    // PairInitialized
    assert!(context.env.emitted_event(
        &context.pair.address(),
        PairInitialized {
            token0: context.token0.address(),
            token1: context.token1.address(),
        }
    ));
}

#[test]
fn test_skim_excess() {
    let mut context = setup();
    let alice = context.env.get_account(1);

    let token0amount = expand_to_18_decimals(3);
    let token1amount = expand_to_18_decimals(3);
    context.add_liquidity(token0amount, token1amount).unwrap();

    let extra_token0 = expand_to_18_decimals(1);
    let extra_token1 = expand_to_18_decimals(2);
    context
        .token0
        .transfer(&context.pair.address(), &extra_token0);
    context
        .token1
        .transfer(&context.pair.address(), &extra_token1);

    context.pair.skim(alice);
    assert!(context.env.emitted_event(
        &context.pair.address(),
        SkimExcess {
            to: alice,
            amount0: extra_token0,
            amount1: extra_token1,
        }
    ));
}

#[test]
fn test_fee_minted() {
    let mut context = setup();
    // ProtocolFeeMinted
    context.factory.set_fee_to(Some(context.alice));
    let token0amount = expand_to_18_decimals(1000);
    let token1amount = expand_to_18_decimals(1000);

    context.add_liquidity(token0amount, token1amount).unwrap();

    let swap_amount = expand_to_18_decimals(1);
    let expected_output_amount = U256::from(996006981039903216_u128);
    context
        .token1
        .transfer(&context.pair.address(), &swap_amount);
    context
        .pair
        .swap(expected_output_amount, U256::zero(), context.owner, None);

    let expected_liquidity = expand_to_18_decimals(1000);
    context.env.set_caller(context.owner);
    context.pair.transfer(
        &context.pair.address(),
        &(expected_liquidity - U256::from(MINIMUM_LIQUIDITY)),
    );

    context.env.set_caller(context.owner);
    context.pair.burn(context.owner);
    dbg!(context.env.event_names(&context.pair.address()));
    assert!(context.env.emitted_event(
        &context.pair.address(),
        ProtocolFeeMinted {
            to: context.alice,
            liquidity: U256::from(249750499251388_u128),
        }
    ));
}

#[test]
fn test_add_liquidity_cspr_refund() {
    let mut context = setup();
    let owner = context.owner;

    // 1. Add initial liquidity to establish price
    // WETH Partner is token, WCSPR is the other.
    // 1000 Partner : 1000 CSPR
    let amount_partner = expand_to_18_decimals(1000);
    let amount_cspr = expand_to_9_decimals(1000);

    context
        .wcspr_partner
        .approve(&context.router.address(), &amount_partner);

    context
        .router
        .with_tokens(amount_cspr.to_u512())
        .add_liquidity_cspr(
            context.wcspr_partner.address(),
            amount_partner,
            amount_partner,
            amount_cspr,
            owner,
            u64::MAX,
        );

    // 2. Add liquidity again with excess CSPR
    // Want to add 100 Partner. Based on 1:1, should take 100 CSPR.
    // We attach 200 CSPR. Should refund 100.
    let amount_partner_2 = expand_to_18_decimals(100);
    let amount_cspr_2_needed = expand_to_9_decimals(100);
    let amount_cspr_2_attached = expand_to_9_decimals(200);

    context
        .wcspr_partner
        .approve(&context.router.address(), &amount_partner_2);

    context
        .router
        .with_tokens(amount_cspr_2_attached.to_u512())
        .add_liquidity_cspr(
            context.wcspr_partner.address(),
            amount_partner_2,
            U256::zero(), // min
            U256::zero(), // min
            owner,
            u64::MAX,
        );

    assert!(context.env.emitted_event(
        &context.router.address(),
        CSPRRefunded {
            to: owner,
            amount: (amount_cspr_2_attached - amount_cspr_2_needed).to_u512(),
        }
    ));
}

#[test]
fn test_swap_cspr_for_exact_tokens_refund() {
    let mut context = setup();
    let owner = context.owner;

    // 1. Add initial liquidity
    let amount_partner = expand_to_18_decimals(1000);
    let amount_cspr = expand_to_9_decimals(1000);

    context
        .wcspr_partner
        .approve(&context.router.address(), &amount_partner);
    context
        .router
        .with_tokens(amount_cspr.to_u512())
        .add_liquidity_cspr(
            context.wcspr_partner.address(),
            amount_partner,
            amount_partner,
            amount_cspr,
            owner,
            u64::MAX,
        );

    // 2. Swap CSPR for exact tokens
    // Want to buy 10 Partner tokens.
    let amount_out = expand_to_18_decimals(10);
    let path = vec![context.wcspr.address(), context.wcspr_partner.address()];

    // Get expected input
    let amounts_in = context.router.get_amounts_in(amount_out, path.clone());
    let amount_in_needed = amounts_in[0];

    // Attach more than needed
    let excess = expand_to_9_decimals(1);
    let amount_in_attached = amount_in_needed + excess;

    context
        .router
        .with_tokens(amount_in_attached.to_u512())
        .swap_cspr_for_exact_tokens(amount_out, path, owner, u64::MAX);

    assert!(context.env.emitted_event(
        &context.router.address(),
        CSPRRefunded {
            to: owner,
            amount: excess.to_u512(),
        }
    ));
}
