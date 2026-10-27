mod common;
use common::{setup, TestContext};
use odra::host::{Deployer, NoArgs};
use odra::prelude::*;
use odra_modules::dapp::events::DappContractAdded;
use odra_modules::dapp::{OwnedDappRegistry, OwnedDappRegistryHostRef};

// The contracts of CSPR.trade know nothing about the dapp registry: the registry lists them, and the
// PairFactory adds every pair the Factory creates.
fn join_dapp(context: &mut TestContext) -> OwnedDappRegistryHostRef {
    let mut registry = OwnedDappRegistry::deploy(&context.env, NoArgs);
    // Only the Factory may create pairs, and the PairFactory adds them to the dapp.
    context
        .pair_factory
        .set_factory_creator(context.factory.address());
    registry.add_dapp_contract(&context.pair_factory.address(), true);
    context
        .pair_factory
        .set_factory_dapp_registry(registry.address());
    // The contracts deployed before join as they are.
    for contract in [
        context.factory.address(),
        context.router.address(),
        context.pair.address(),
    ] {
        registry.add_dapp_contract(&contract, false);
    }
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
