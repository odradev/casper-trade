mod common;
use common::{setup, TestContext};
use odra::host::{Deployer, HostRef, NoArgs};
use odra::prelude::*;
use odra_modules::dapp::errors::Error;
use odra_modules::dapp::events::DappContractAdded;
use odra_modules::dapp::{DappContractHostRef, OwnedDappRegistry, OwnedDappRegistryHostRef};

// Factory and Router point to the registry from their init. The PairFactory points to it once
// configured, and then points every pair it creates or upgrades there too; it adds the pairs it creates.
fn join_dapp(context: &mut TestContext) -> OwnedDappRegistryHostRef {
    let mut registry =
        OwnedDappRegistryHostRef::new(context.dapp_registry.address(), context.env.clone());
    // Only the Factory may create pairs.
    context
        .pair_factory
        .set_factory_creator(context.factory.address());
    context
        .pair_factory
        .set_factory_dapp_registry(registry.address());
    registry.add_dapp_contract(&context.pair_factory.address(), true);
    registry.add_dapp_contract(&context.factory.address(), false);
    registry.add_dapp_contract(&context.router.address(), false);
    // The pair created before points to the registry after an upgrade.
    let symbol = context.pair.symbol();
    context.pair_factory.upgrade_child_contract(symbol);
    registry.add_dapp_contract(&context.pair.address(), false);
    registry
}

#[test]
fn created_pairs_join_the_dapp() {
    let mut context = setup();
    let registry = join_dapp(&mut context);

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
            registrar: context.pair_factory.address(),
        }
    ));
    assert_eq!(
        registry.get_dapp_contracts(),
        vec![
            registry.address(),
            context.pair_factory.address(),
            context.factory.address(),
            context.router.address(),
            context.pair.address(),
            new_pair,
        ]
    );
    assert!(registry.is_dapp_registrar(&context.pair_factory.address()));

    // Every contract points to the registry; the pairs need no code of their own.
    for contract in registry.get_dapp_contracts() {
        let contract = DappContractHostRef::new(contract, context.env.clone());
        assert_eq!(contract.get_dapp_registry(), registry.address());
    }
}

// A pair the PairFactory created before it had a registry does not point to it until upgraded.
#[test]
fn old_pairs_join_after_an_upgrade() {
    let mut context = setup();
    let mut registry =
        OwnedDappRegistryHostRef::new(context.dapp_registry.address(), context.env.clone());
    assert_eq!(
        registry.try_add_dapp_contract(&context.pair.address(), false),
        Err(ExecutionError::DappRegistryNotSet.into())
    );
    let other = OwnedDappRegistry::deploy(&context.env, NoArgs);
    assert_eq!(
        registry.try_add_dapp_contract(&other.address(), false),
        Err(Error::DappRegistryMismatch.into())
    );
    join_dapp(&mut context);
    assert!(registry.is_dapp_contract(&context.pair.address()));
}

// Pairs are created only through the Factory, so nobody else can add contracts to the dapp.
#[test]
fn only_the_factory_creates_pairs() {
    let mut context = setup();
    let registry = join_dapp(&mut context);
    let (factory, token0, token1) = (
        context.factory.address(),
        context.token0.address(),
        context.wcspr_partner.address(),
    );

    for caller in [context.owner, context.alice] {
        context.env.set_caller(caller);
        assert_eq!(
            context
                .pair_factory
                .try_new_contract("Fake".to_string(), factory, token0, token1),
            Err(ExecutionError::CallerNotFactoryCreator.into())
        );
    }
    assert_eq!(registry.get_dapp_contracts().len(), 5);
}

// Only the deployer of the PairFactory may pick its creator and registry.
#[test]
fn only_the_deployer_configures_the_pair_factory() {
    let mut context = setup();
    let registry = OwnedDappRegistry::deploy(&context.env, NoArgs);

    // A registry needs a creator first.
    assert_eq!(
        context
            .pair_factory
            .try_set_factory_dapp_registry(registry.address()),
        Err(ExecutionError::FactoryCreatorNotSet.into())
    );

    context.env.set_caller(context.alice);
    assert!(context
        .pair_factory
        .try_set_factory_creator(context.alice)
        .is_err());
}
