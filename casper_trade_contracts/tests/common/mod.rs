//! Common test utilities for integration tests
//!
//! This module re-exports or provides test setup functions
//! that can be shared across multiple integration test files.

use casper_trade_contracts::factory::{Factory, FactoryHostRef, FactoryInitArgs};
use casper_trade_contracts::pair::PairFactory;
use odra::host::{Deployer, HostEnv, NoArgs};
use odra::prelude::*;

pub struct TestContext {
    pub env: HostEnv,
    pub factory: FactoryHostRef,
    pub deployer: Address,
}

pub fn setup() -> TestContext {
    let env = odra_test::env();
    let deployer = env.get_account(0);
    let pair_factory = PairFactory::deploy(&env, NoArgs);

    let factory = Factory::deploy(
        &env,
        FactoryInitArgs {
            fee_to: None,
            pair_factory: pair_factory.address(),
        },
    );

    TestContext {
        env,
        factory,
        deployer,
    }
}

