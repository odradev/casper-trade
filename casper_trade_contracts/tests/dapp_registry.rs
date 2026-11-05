mod common;
use casper_trade_contracts::factory::{Factory, FactoryUpgradeArgs};
use casper_trade_contracts::router::{Router, RouterUpgradeArgs};
use common::setup;
use odra::host::{Deployer, HostRef, NoArgs};
use odra::prelude::*;
use odra_modules::access::errors::Error as AccessError;
use odra_modules::dapp::errors::Error;
use odra_modules::dapp::events::{DappContractAdded, DappRegistryChanged};
use odra_modules::dapp::{DappContractHostRef, OwnedDappRegistry, OwnedDappRegistryHostRef};

// The Factory is a registrar of the dapp: it passes the registry to every pair it creates and adds the
// pair to it.
#[test]
fn created_pairs_join_the_dapp() {
    let mut context = setup();
    let mut registry =
        OwnedDappRegistryHostRef::new(context.dapp_registry.address(), context.env.clone());
    registry.add_dapp_contract(&context.router.address(), false);

    let new_pair = context
        .factory
        .create_pair(context.token0.address(), context.wcspr.address());
    assert!(registry.is_dapp_contract(&new_pair));
    assert!(!registry.is_dapp_registrar(&new_pair));
    assert!(context.env.emitted_event(
        &registry,
        DappContractAdded {
            contract: new_pair,
            is_registrar: false,
            registrar: context.factory.address(),
        }
    ));
    assert_eq!(
        registry.get_dapp_contracts(),
        vec![
            registry.address(),
            context.factory.address(),
            context.pair.address(),
            context.router.address(),
            new_pair,
        ]
    );
    assert!(registry.is_dapp_registrar(&context.factory.address()));

    // Every contract points to the registry.
    for contract in registry.get_dapp_contracts() {
        let contract = DappContractHostRef::new(contract, context.env.clone());
        assert_eq!(contract.get_dapp_registry(), registry.address());
    }
}

// A pair created directly through the PairFactory cannot join: only the Factory registers pairs.
#[test]
fn pairs_created_elsewhere_do_not_join() {
    let mut context = setup();
    let mut registry =
        OwnedDappRegistryHostRef::new(context.dapp_registry.address(), context.env.clone());
    context.env.set_caller(context.alice);
    let (fake, _) = context.pair_factory.new_contract(
        "Fake".to_string(),
        context.factory.address(),
        context.token0.address(),
        context.wcspr_partner.address(),
        registry.address(),
    );
    assert_eq!(
        registry.try_add_dapp_contract(&fake, false),
        Err(AccessError::CallerNotTheOwner.into())
    );
    assert!(!registry.is_dapp_contract(&fake));
}

// The Factory creates pairs only while it is a registrar of the registry it points to.
#[test]
fn the_factory_must_be_a_registrar() {
    let mut context = setup();
    let other = OwnedDappRegistry::deploy(&context.env, NoArgs);
    context.factory.set_dapp_registry(&other.address());
    assert_eq!(
        context
            .factory
            .try_create_pair(context.token0.address(), context.wcspr.address()),
        Err(Error::CallerNotDappRegistrar.into())
    );
}

// `upgrade(None)` keeps the registry of the Factory and the Router; `upgrade(Some(..))` moves them.
#[test]
fn factory_and_router_upgrades_keep_or_move_the_registry() {
    let context = setup();
    let env = &context.env;
    let registry = context.dapp_registry.address();
    let other = OwnedDappRegistry::deploy(env, NoArgs);

    let factory = context.factory.address();
    let router = context.router.address();
    Factory::try_upgrade(
        env,
        factory,
        FactoryUpgradeArgs {
            dapp_registry: None,
        },
    )
    .unwrap();
    Router::try_upgrade(
        env,
        router,
        RouterUpgradeArgs {
            dapp_registry: None,
        },
    )
    .unwrap();
    assert_eq!(context.factory.get_dapp_registry(), registry);
    assert_eq!(context.router.get_dapp_registry(), registry);

    let moved = FactoryUpgradeArgs {
        dapp_registry: Some(other.address()),
    };
    Factory::try_upgrade(env, factory, moved).unwrap();
    let moved = RouterUpgradeArgs {
        dapp_registry: Some(other.address()),
    };
    Router::try_upgrade(env, router, moved).unwrap();
    assert_eq!(context.factory.get_dapp_registry(), other.address());
    assert_eq!(context.router.get_dapp_registry(), other.address());
}

// On upgrade a pair takes the registry its Factory points to, whatever revision it is at: a pair from
// before the dapp registry joins it, and a forced migration moves pairs already in a registry along
// with the Factory.
#[test]
fn upgraded_pairs_follow_the_factory_registry() {
    let mut context = setup();
    let env = context.env.clone();
    let previous = context.dapp_registry.address();
    let mut other = OwnedDappRegistry::deploy(&env, NoArgs);

    let moved = FactoryUpgradeArgs {
        dapp_registry: Some(other.address()),
    };
    Factory::try_upgrade(&env, context.factory.address(), moved).unwrap();
    context
        .pair_factory
        .upgrade_child_contract(context.pair.symbol());

    assert_eq!(context.pair.get_dapp_registry(), other.address());
    assert!(env.emitted_event(
        &context.pair,
        DappRegistryChanged {
            previous_registry: Some(previous),
            new_registry: other.address(),
        }
    ));
    // The pair points back to the new registry, so it can join it there.
    other.add_dapp_contract(&context.factory.address(), true);
    other.add_dapp_contract(&context.pair.address(), false);
    assert!(other.is_dapp_contract(&context.pair.address()));
}

// An upgrade of a pair whose Factory stayed with its registry changes nothing and emits nothing.
#[test]
fn upgrading_a_pair_in_place_keeps_its_registry() {
    let mut context = setup();
    let registry = context.dapp_registry.address();
    let events_before = context.env.events_count(&context.pair);

    context
        .pair_factory
        .upgrade_child_contract(context.pair.symbol());

    assert_eq!(context.pair.get_dapp_registry(), registry);
    assert_eq!(context.env.events_count(&context.pair), events_before);
}
