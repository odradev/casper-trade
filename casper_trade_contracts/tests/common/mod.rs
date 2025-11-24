//! Common test utilities for integration tests
//!
//! This module re-exports or provides test setup functions
//! that can be shared across multiple integration test files.

use casper_trade_contracts::factory::{Factory, FactoryHostRef, FactoryInitArgs};
use casper_trade_contracts::pair::{PairFactory, PairHostRef};
use casper_trade_contracts::router::{Router, RouterHostRef, RouterInitArgs};
use casper_trade_contracts::sample_tokens::{SampleToken, SampleTokenHostRef, SampleTokenInitArgs};
use casper_trade_contracts::utils::expand_to_18_decimals;
use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, HostRef, NoArgs};
use odra::prelude::*;
use odra_modules::cep18_token::Cep18HostRef;
use odra_modules::wrapped_native::{WrappedNativeToken, WrappedNativeTokenHostRef};

pub struct TestContext {
    pub env: HostEnv,
    pub factory: FactoryHostRef,
    pub router: RouterHostRef,
    pub token0: SampleTokenHostRef,
    pub token1: SampleTokenHostRef,
    pub wcspr: WrappedNativeTokenHostRef,
    pub wcspr_partner: SampleTokenHostRef,
    pub pair: PairHostRef,
    pub wcspr_pair: PairHostRef,
    pub owner: Address,
    pub alice: Address,
    pub bob: Address,
}

pub fn setup() -> TestContext {
    let env = odra_test::env();
    let owner = env.get_account(0);
    let alice = env.get_account(1);
    let bob = env.get_account(2);
    let pair_factory = PairFactory::deploy(&env, NoArgs);

    // Deploy the actual Factory contract
    let mut factory = Factory::deploy(
        &env,
        FactoryInitArgs {
            fee_to: None,
            pair_factory: pair_factory.address(),
        },
    );

    // Deploy WCSPR contract
    let wcspr = WrappedNativeToken::deploy(&env, NoArgs);

    // Deploy Router with the factory and wcspr address
    let router = Router::deploy(
        &env,
        RouterInitArgs {
            factory: factory.address(),
            wcspr: wcspr.address(),
        },
    );

    // Deploy tokens
    let token0 = SampleToken::deploy(
        &env,
        SampleTokenInitArgs {
            name: "Sample Token A".to_string(),
            symbol: "STA".to_string(),
            decimals: 18,
            initial_supply: expand_to_18_decimals(10000),
        },
    );

    let token1 = SampleToken::deploy(
        &env,
        SampleTokenInitArgs {
            name: "Sample Token B".to_string(),
            symbol: "STB".to_string(),
            decimals: 18,
            initial_supply: expand_to_18_decimals(10000),
        },
    );

    // Deploy WCSPR partner token
    let wcspr_partner = SampleToken::deploy(
        &env,
        SampleTokenInitArgs {
            name: "WETH Partner".to_string(),
            symbol: "WETHP".to_string(),
            decimals: 18,
            initial_supply: expand_to_18_decimals(10000),
        },
    );

    // Create pairs via factory
    let pair_address = factory.create_pair(token0.address(), token1.address());
    let pair = PairHostRef::new(pair_address, env.clone());
    let wcspr_pair_address = factory.create_pair(wcspr.address(), wcspr_partner.address());
    let wcspr_pair = PairHostRef::new(wcspr_pair_address, env.clone());

    TestContext {
        env,
        factory,
        router,
        token0,
        token1,
        wcspr,
        wcspr_partner,
        pair,
        wcspr_pair,
        owner,
        alice,
        bob,
    }
}

impl TestContext {
    pub fn add_liquidity(&mut self, token0_amount: U256, token1_amount: U256) -> OdraResult<U256> {
        self.env.set_caller(self.owner);
        let pair_address = self.pair.address();
        let mut pair_instance = PairHostRef::new(pair_address, self.env.clone());
        let mut token0_instance = Cep18HostRef::new(pair_instance.token0(), self.env.clone());
        let mut token1_instance = Cep18HostRef::new(pair_instance.token1(), self.env.clone());
        token0_instance.transfer(&pair_instance.address(), &token0_amount);
        token1_instance.transfer(&pair_instance.address(), &token1_amount);

        pair_instance.try_mint(self.owner)
    }
}
