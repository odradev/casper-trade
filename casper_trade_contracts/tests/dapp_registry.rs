mod common;
use casper_trade_contracts::factory::{Factory, FactoryUpgradeArgs};
use casper_trade_contracts::router::{Router, RouterHostRef, RouterInitArgs};
use common::{setup, TestContext};
use odra::casper_types::bytesrepr::FromBytes;
use odra::host::{Deployer, HostRef, NoArgs};
use odra::prelude::*;
use odra_modules::access::errors::Error as AccessError;
use odra_modules::dapp::errors::Error;
use odra_modules::dapp::events::{DappContractAdded, DappRegistryChanged};
use odra_modules::dapp::storage::{CONTRACTS_KEY, DAPP_KEY, REGISTRARS_KEY, REGISTRY_KEY};
use odra_modules::dapp::{DappContractHostRef, DappMetadata};

/// The registry of another dapp: a second router.
fn another_registry(context: &TestContext) -> RouterHostRef {
    Router::deploy(
        &context.env,
        RouterInitArgs {
            factory: context.factory.address(),
            wcspr: context.wcspr.address(),
        },
    )
}

/// The registry a contract points to, read from the `registry` item of its `__dapp` dictionary.
fn dapp_registry_of(context: &TestContext, contract: &Address) -> Option<Address> {
    let bytes = context
        .env
        .get_dictionary_value(contract, DAPP_KEY, REGISTRY_KEY.as_bytes())?;
    Address::from_bytes(&bytes).ok().map(|(address, _)| address)
}

/// A list item of the `__dapp` dictionary of a registry, read from the chain.
fn dapp_list(context: &TestContext, registry: &Address, key: &str) -> Option<Vec<Address>> {
    let bytes = context
        .env
        .get_dictionary_value(registry, DAPP_KEY, key.as_bytes())?;
    Vec::<Address>::from_bytes(&bytes)
        .ok()
        .map(|(list, _)| list)
}

fn metadata() -> DappMetadata {
    DappMetadata {
        name: "CSPR.trade".to_string(),
        description: "Automated Market Maker on Casper Network".to_string(),
        website_url: "https://cspr.trade".to_string(),
        icon_url: "https://cspr.trade/icon.png".to_string(),
    }
}

// The Router is the dapp registry: it lists itself first, points to itself like every contract of the
// dapp, and its deployer owns it.
#[test]
fn the_router_is_the_registry() {
    let mut context = setup();
    let router = context.router.address();
    assert_eq!(context.router.get_dapp_registry(), router);
    assert_eq!(dapp_registry_of(&context, &router), Some(router));
    assert_eq!(context.router.get_dapp_contracts()[0], router);
    assert!(context.router.is_dapp_contract(&router));
    assert_eq!(context.router.get_owner(), context.owner);
    // The registry lists are stored from the start, so the chain tells the registry from a member
    // before anything is added (odra 2.10.1); here the Factory and the pair it created are in.
    assert_eq!(
        dapp_list(&context, &router, CONTRACTS_KEY),
        Some(vec![context.factory.address(), context.pair.address()])
    );
    assert_eq!(
        dapp_list(&context, &router, REGISTRARS_KEY),
        Some(vec![context.factory.address()])
    );
    let fresh = another_registry(&context).address();
    assert_eq!(dapp_list(&context, &fresh, CONTRACTS_KEY), Some(vec![]));
    assert_eq!(dapp_list(&context, &fresh, REGISTRARS_KEY), Some(vec![]));

    // The owner sets the dapp metadata.
    assert_eq!(context.router.get_dapp_metadata(), DappMetadata::default());
    context.router.set_dapp_metadata(metadata());
    assert_eq!(context.router.get_dapp_metadata(), metadata());
    context.env.set_caller(context.alice);
    assert_eq!(
        context
            .router
            .try_set_dapp_metadata(DappMetadata::default()),
        Err(AccessError::CallerNotTheOwner.into())
    );
}

// An upgrade of a Router that is the registry already changes nothing: the owner, the metadata and
// the contracts stay, and no event is emitted.
#[test]
fn upgrading_the_router_keeps_the_registry() {
    let mut context = setup();
    let router = context.router.address();
    context.router.set_dapp_metadata(metadata());
    let contracts = context.router.get_dapp_contracts();
    let events_before = context.env.events_count(&context.router);

    Router::try_upgrade(&context.env, router, NoArgs).unwrap();

    assert_eq!(context.router.get_owner(), context.owner);
    assert_eq!(context.router.get_dapp_metadata(), metadata());
    assert_eq!(context.router.get_dapp_contracts(), contracts);
    assert_eq!(context.router.get_dapp_registry(), router);
    assert_eq!(dapp_registry_of(&context, &router), Some(router));
    assert_eq!(context.env.events_count(&context.router), events_before);
}

// The Factory is a registrar of the dapp: it passes the registry to every pair it creates and adds the
// pair to it.
#[test]
fn created_pairs_join_the_dapp() {
    let mut context = setup();
    let registry = context.router.address();

    let new_pair = context
        .factory
        .create_pair(context.token0.address(), context.wcspr.address());
    assert!(context.router.is_dapp_contract(&new_pair));
    assert!(!context.router.is_dapp_registrar(&new_pair));
    assert!(context.env.emitted_event(
        &context.router,
        DappContractAdded {
            contract: new_pair,
            is_registrar: false,
            registrar: context.factory.address(),
        }
    ));
    assert_eq!(
        context.router.get_dapp_contracts(),
        vec![
            registry,
            context.factory.address(),
            context.pair.address(),
            new_pair,
        ]
    );
    assert!(context.router.is_dapp_registrar(&context.factory.address()));

    // Every contract points to the registry, the Router itself included.
    for contract in context.router.get_dapp_contracts() {
        let dapp_contract = DappContractHostRef::new(contract, context.env.clone());
        assert_eq!(dapp_contract.get_dapp_registry(), registry);
        assert_eq!(dapp_registry_of(&context, &contract), Some(registry));
    }
}

// A pair created directly through the PairFactory cannot join: only the Factory registers pairs.
#[test]
fn pairs_created_elsewhere_do_not_join() {
    let mut context = setup();
    context.env.set_caller(context.alice);
    let (fake, _) = context.pair_factory.new_contract(
        "Fake".to_string(),
        context.factory.address(),
        context.token0.address(),
        context.wcspr_partner.address(),
        context.router.address(),
    );
    assert_eq!(
        context.router.try_add_dapp_contract(&fake, false),
        Err(AccessError::CallerNotTheOwner.into())
    );
    assert!(!context.router.is_dapp_contract(&fake));
}

// The Factory creates pairs only while it is a registrar of the registry it points to.
#[test]
fn the_factory_must_be_a_registrar() {
    let mut context = setup();
    let other = another_registry(&context);
    context.factory.set_dapp_registry(&other.address());
    assert_eq!(
        context
            .factory
            .try_create_pair(context.token0.address(), context.wcspr.address()),
        Err(Error::CallerNotDappRegistrar.into())
    );
}

// `upgrade(None)` keeps the registry of the Factory; `upgrade(Some(..))` moves it.
#[test]
fn factory_upgrades_keep_or_move_the_registry() {
    let context = setup();
    let env = &context.env;
    let registry = context.router.address();
    let other = another_registry(&context);

    let factory = context.factory.address();
    Factory::try_upgrade(
        env,
        factory,
        FactoryUpgradeArgs {
            dapp_registry: None,
        },
    )
    .unwrap();
    assert_eq!(context.factory.get_dapp_registry(), registry);

    let moved = FactoryUpgradeArgs {
        dapp_registry: Some(other.address()),
    };
    Factory::try_upgrade(env, factory, moved).unwrap();
    assert_eq!(context.factory.get_dapp_registry(), other.address());
}

// On upgrade a pair takes the registry its Factory points to, whatever revision it is at: a pair from
// before the dapp registry joins it, and a forced migration moves pairs already in a registry along
// with the Factory.
#[test]
fn upgraded_pairs_follow_the_factory_registry() {
    let mut context = setup();
    let env = context.env.clone();
    let previous = context.router.address();
    let mut other = another_registry(&context);

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
    let registry = context.router.address();
    let events_before = context.env.events_count(&context.pair);

    context
        .pair_factory
        .upgrade_child_contract(context.pair.symbol());

    assert_eq!(context.pair.get_dapp_registry(), registry);
    assert_eq!(context.env.events_count(&context.pair), events_before);
}
