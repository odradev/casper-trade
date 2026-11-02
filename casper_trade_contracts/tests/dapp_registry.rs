mod common;
use common::setup;
use odra::host::{Deployer, HostRef, NoArgs};
use odra::prelude::*;
use odra_modules::access::errors::Error as AccessError;
use odra_modules::dapp::errors::Error;
use odra_modules::dapp::events::DappContractAdded;
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
