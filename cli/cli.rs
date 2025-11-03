//! CLI tool for Casper Trade smart contracts
//!
//! This CLI provides deployment and interaction capabilities for the Casper Trade DEX contracts.

use casper_trade_contracts::casper_trade_v2_pair::{
    CasperTradeV2Pair, CasperTradeV2PairFactory, CasperTradeV2PairFactoryHostRef,
    CasperTradeV2PairInitArgs,
};
use casper_trade_contracts::factory::{Factory, FactoryInitArgs};
use casper_trade_contracts::router::{CasperTradeV2Router, CasperTradeV2RouterInitArgs};
use casper_trade_contracts::sample_tokens::{SampleToken, SampleTokenInitArgs};
use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, InstallConfig, NoArgs};
use odra::prelude::Addressable;
use odra_cli::{cspr, deploy::DeployScript, DeployedContractsContainer, DeployerExt, OdraCli};
use odra_modules::wrapped_native::WrappedNativeToken;

mod scenarios;
use scenarios::{AddLiquidity, MintTokens, SetupPair, SwapTokens};

use crate::scenarios::AddLiquidityCSPR;

/// Deploys all Casper Trade contracts
pub struct ContractsDeployScript;

impl DeployScript for ContractsDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer,
    ) -> Result<(), odra_cli::deploy::Error> {
        env.set_gas(50_000_000_000);

        let pair_factory = CasperTradeV2PairFactory::deploy(&env, NoArgs);

        // Deploy Factory contract
        let mut factory = Factory::load_or_deploy_with_cfg(
            env,
            None,
            FactoryInitArgs {
                fee_to: Some(env.get_account(0)), // Set deployer as fee collector
                pair_factory: pair_factory.address(),
            },
            InstallConfig::upgradable::<Factory>(),
            container,
            cspr!(400),
        )?;

        println!("Factory deployed successfully!");

        // Deploy Sample Token A
        let token_a = SampleToken::load_or_deploy_with_cfg(
            env,
            Some("SampleTokenA".to_string()),
            SampleTokenInitArgs {
                name: "Sample Token A".to_string(),
                symbol: "TKNA".to_string(),
                decimals: 18,
                initial_supply: U256::from(1_000_000_000u64) * U256::exp10(18), // 1 billion tokens
            },
            InstallConfig {
                package_named_key: "sample_token_a".to_string(),
                is_upgradable: true,
                allow_key_override: true,
            },
            container,
            cspr!(400),
        )?;

        println!("Sample Token A deployed successfully!");
        println!("  Name: {}", token_a.name());
        println!("  Symbol: {}", token_a.symbol());
        println!("  Total Supply: {}", token_a.total_supply());

        // Deploy Sample Token B
        let token_b = SampleToken::load_or_deploy_with_cfg(
            env,
            Some("SampleTokenB".to_string()),
            SampleTokenInitArgs {
                name: "Sample Token B".to_string(),
                symbol: "TKNB".to_string(),
                decimals: 18,
                initial_supply: U256::from(1_000_000_000u64) * U256::exp10(18), // 1 billion tokens
            },
            InstallConfig {
                package_named_key: "sample_token_b".to_string(),
                is_upgradable: true,
                allow_key_override: true,
            },
            container,
            cspr!(400),
        )?;

        println!("Sample Token B deployed successfully!");
        println!("  Name: {}", token_b.name());
        println!("  Symbol: {}", token_b.symbol());
        println!("  Total Supply: {}", token_b.total_supply());

        // Deploy Wrapped Native Token (WCSPR)
        let wcspr = WrappedNativeToken::load_or_deploy_with_cfg(
            env,
            None,
            NoArgs,
            InstallConfig::upgradable::<WrappedNativeToken>(),
            container,
            cspr!(500),
        )?;

        println!("Wrapped Native Token (WCSPR) deployed successfully!");

        // Deploy Router
        let router = CasperTradeV2Router::load_or_deploy_with_cfg(
            env,
            None,
            CasperTradeV2RouterInitArgs {
                factory: factory.address(),
                wcspr: wcspr.address(),
            },
            InstallConfig::upgradable::<CasperTradeV2Router>(),
            container,
            cspr!(500),
        )?;

        println!("Router deployed successfully!");
        println!("  Factory: {:?}", router.factory_address());
        println!("  WCSPR: {:?}", router.wcspr());

        // Deploy and initialize trading pairs
        println!("\nDeploying trading pairs...");

        // Deploy TokenA-TokenB pair
        let mut pair_a_b = CasperTradeV2Pair::load_or_deploy_with_cfg(
            env,
            Some("TokenA_TokenB".to_string()),
            CasperTradeV2PairInitArgs {
                factory: factory.address(),
            },
            InstallConfig::upgradable::<CasperTradeV2Pair>(),
            container,
            cspr!(500),
        )?;
        pair_a_b.initialize(token_a.address(), token_b.address());
        println!("  ✓ TokenA-TokenB pair deployed and initialized");

        // Deploy TokenA-WCSPR pair
        let mut pair_a_wcspr = CasperTradeV2Pair::load_or_deploy_with_cfg(
            env,
            Some("TokenA_WCSPR".to_string()),
            CasperTradeV2PairInitArgs {
                factory: factory.address(),
            },
            InstallConfig::upgradable::<CasperTradeV2Pair>(),
            container,
            cspr!(500),
        )?;
        pair_a_wcspr.initialize(token_a.address(), wcspr.address());
        println!("  ✓ TokenA-WCSPR pair deployed and initialized");

        // Deploy TokenB-WCSPR pair
        let mut pair_b_wcspr = CasperTradeV2Pair::load_or_deploy_with_cfg(
            env,
            Some("TokenB_WCSPR".to_string()),
            CasperTradeV2PairInitArgs {
                factory: factory.address(),
            },
            InstallConfig::upgradable::<CasperTradeV2Pair>(),
            container,
            cspr!(500),
        )?;
        pair_b_wcspr.initialize(token_b.address(), wcspr.address());
        println!("  ✓ TokenB-WCSPR pair deployed and initialized");

        println!("\n✓ Deployment completed successfully!");
        println!("\nDeployed contracts:");
        println!("  - Factory");
        println!("  - SampleTokenA (TKNA)");
        println!("  - SampleTokenB (TKNB)");
        println!("  - WrappedNativeToken (WCSPR)");
        println!("  - Router");
        println!("\nTrading pairs:");
        println!("  - TokenA_TokenB");
        println!("  - TokenA_WCSPR");
        println!("  - TokenB_WCSPR");
        println!("\nNext steps:");
        println!("  1. Use 'mint-tokens' to mint tokens to accounts");
        println!("  2. Use 'add-liquidity' to add liquidity to pairs");
        println!("  3. Use 'swap-tokens' to swap tokens");

        Ok(())
    }
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("Casper Trade CLI - Automated Market Maker on Casper Network")
        .deploy(ContractsDeployScript)
        .contract::<Factory>()
        .contract::<CasperTradeV2Router>()
        .contract::<WrappedNativeToken>()
        .contract::<CasperTradeV2PairFactory>()
        .named_contract::<SampleToken>("SampleTokenA".to_string())
        .named_contract::<SampleToken>("SampleTokenB".to_string())
        .named_contract::<CasperTradeV2Pair>("TokenA_TokenB".to_string())
        .named_contract::<CasperTradeV2Pair>("TokenA_WCSPR".to_string())
        .named_contract::<CasperTradeV2Pair>("TokenB_WCSPR".to_string())
        .scenario(MintTokens)
        .scenario(SetupPair)
        .scenario(AddLiquidity)
        .scenario(AddLiquidityCSPR)
        .scenario(SwapTokens)
        .build()
        .run();
}
