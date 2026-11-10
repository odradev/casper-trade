//! Common test utilities for integration tests
//!
//! This module re-exports or provides test setup functions
//! that can be shared across multiple integration test files.

use casper_trade_contracts::factory::{Factory, FactoryHostRef, FactoryInitArgs};
use casper_trade_contracts::pair::{PairFactory, PairFactoryHostRef, PairHostRef};
use casper_trade_contracts::router::{Router, RouterHostRef, RouterInitArgs};
use casper_trade_contracts::sample_tokens::{SampleToken, SampleTokenHostRef, SampleTokenInitArgs};
use casper_trade_contracts::utils::{expand_to_18_decimals, sort_tokens};
use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, HostRef, InstallConfig, NoArgs};
use odra::prelude::*;
use odra_modules::cep18_token::Cep18HostRef;
use odra_modules::wrapped_native::{WrappedNativeToken, WrappedNativeTokenHostRef};

#[allow(dead_code)]
pub struct TestContext {
    pub env: HostEnv,
    pub factory: FactoryHostRef,
    pub pair_factory: PairFactoryHostRef,
    /// The router is also the dapp registry.
    pub router: RouterHostRef,
    pub token0: SampleTokenHostRef,
    pub token1: SampleTokenHostRef,
    pub wcspr: WrappedNativeTokenHostRef,
    pub wcspr_partner: SampleTokenHostRef,
    pub pair: PairHostRef,
    pub owner: Address,
    pub alice: Address,
}

#[allow(dead_code)]
pub fn setup() -> TestContext {
    let env = odra_test::env();
    let owner = env.get_account(0);
    let alice = env.get_account(1);
    let pair_factory = PairFactory::deploy(&env, NoArgs);

    // Deploy the actual Factory contract; upgradable like on a live network, so the tests can upgrade it.
    let mut factory = Factory::deploy_with_cfg(
        &env,
        FactoryInitArgs {
            fee_to: None,
            pair_factory: pair_factory.address(),
        },
        InstallConfig::upgradable::<Factory>(),
    );

    // Deploy WCSPR contract
    let wcspr = WrappedNativeToken::deploy(&env, NoArgs);

    // Deploy Router with the factory and wcspr address; upgradable like the Factory. The router is the
    // dapp registry.
    let mut router = Router::deploy_with_cfg(
        &env,
        RouterInitArgs {
            factory: factory.address(),
            wcspr: wcspr.address(),
        },
        InstallConfig::upgradable::<Router>(),
    );
    // The factory joins the dapp as a registrar: it registers the pairs it creates.
    factory.set_dapp_registry(&router.address());
    router.add_dapp_contract(&factory.address(), true);

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

    // The tests expect the tokens in the order the factory sorts them.
    let (token0, token1) = sort_tokens(token0, token1);

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

    TestContext {
        env,
        factory,
        pair_factory,
        router,
        token0,
        token1,
        wcspr,
        wcspr_partner,
        pair,
        owner,
        alice,
    }
}

impl TestContext {
    #[allow(dead_code)]
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
