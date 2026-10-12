mod common;

use common::setup;
use odra::prelude::*;
use odra_modules::dapp::events::DappContractAdded;

// The factory is a dapp factory: every pair it creates points to the registry and is registered in it.
#[test]
fn created_pairs_belong_to_the_dapp() {
    let mut context = setup();
    let registry = context.dapp_registry.address();

    assert_eq!(context.factory.get_dapp_registry(), registry);
    assert_eq!(context.router.get_dapp_registry(), registry);
    assert_eq!(context.pair.get_dapp_registry(), registry);
    assert!(context
        .dapp_registry
        .is_dapp_factory(&context.factory.address()));

    // The pair created in the setup is registered by the factory.
    assert!(context
        .dapp_registry
        .is_dapp_contract(&context.pair.address()));
    assert!(!context
        .dapp_registry
        .is_dapp_factory(&context.pair.address()));
    assert!(context.env.emitted_event(
        &context.dapp_registry,
        DappContractAdded {
            contract: context.pair.address(),
            is_factory: false,
            registrar: context.factory.address(),
        }
    ));

    // A pair created later joins the dapp as well.
    let new_pair = context
        .factory
        .create_pair(context.token0.address(), context.wcspr.address());
    assert!(context.dapp_registry.is_dapp_contract(&new_pair));

    // The owner registers the router.
    context
        .dapp_registry
        .add_dapp_contract(&context.router.address(), false);

    assert_eq!(
        context.dapp_registry.get_dapp_contracts(),
        vec![
            registry,
            context.factory.address(),
            context.pair.address(),
            new_pair,
            context.router.address(),
        ]
    );
}

// Only an admin of the factory may point it to another registry.
#[test]
fn only_admin_sets_factory_registry() {
    let mut context = setup();
    let other_registry = context.router.address();

    context.env.set_caller(context.alice);
    assert!(context
        .factory
        .try_set_dapp_registry(&other_registry)
        .is_err());

    context.env.set_caller(context.owner);
    context.factory.set_dapp_registry(&other_registry);
    assert_eq!(context.factory.get_dapp_registry(), other_registry);
}

// The PairFactory joins the dapp as a regular contract; the pairs it deploys keep answering with
// their own registry, although they share the factory's `get_dapp_registry`.
#[test]
fn pair_factory_belongs_to_the_dapp() {
    let mut context = setup();
    let registry = context.dapp_registry.address();

    // Only the deployer holds the factory access.
    context.env.set_caller(context.alice);
    assert!(context
        .pair_factory
        .try_set_factory_dapp_registry(registry)
        .is_err());

    context.env.set_caller(context.owner);
    context.pair_factory.set_factory_dapp_registry(registry);
    assert_eq!(context.pair_factory.get_dapp_registry(), registry);
    context
        .dapp_registry
        .add_dapp_contract(&context.pair_factory.address(), false);
    assert!(context
        .dapp_registry
        .is_dapp_contract(&context.pair_factory.address()));

    assert_eq!(context.pair.get_dapp_registry(), registry);
    let new_pair = context
        .factory
        .create_pair(context.token1.address(), context.wcspr.address());
    assert!(context.dapp_registry.is_dapp_contract(&new_pair));
}
