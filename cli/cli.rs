//! CLI tool for Casper Trade smart contracts
//!
//! This CLI provides deployment and interaction capabilities for the Casper Trade DEX contracts.

use casper_trade_contracts::factory::{Factory, FactoryInitArgs};
use casper_trade_contracts::pair::{Pair, PairFactory, PairHostRef};
use casper_trade_contracts::router::{Router, RouterInitArgs};
use casper_trade_contracts::sample_tokens::{SampleToken, SampleTokenInitArgs};
use odra::casper_types::U256;
use odra::host::{HostEnv, HostRef, InstallConfig, NoArgs};
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
        let balance_before = env.balance_of(&env.caller());
        env.set_gas(800_000_000_000);

        let pair_factory = PairFactory::load_or_deploy_with_cfg(
            env,
            None,
            NoArgs,
            InstallConfig::upgradable::<PairFactory>(),
            container,
            cspr!(800),
        )?;

        env.set_gas(300_000_000_000);
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
            cspr!(400),
        )?;

        println!("Wrapped Native Token (WCSPR) deployed successfully!");

        // Deploy Router
        let router = Router::load_or_deploy_with_cfg(
            env,
            None,
            RouterInitArgs {
                factory: factory.address(),
                wcspr: wcspr.address(),
            },
            InstallConfig::upgradable::<Router>(),
            container,
            cspr!(500),
        )?;

        println!("Router deployed successfully!");
        println!("  Factory: {:?}", router.factory_address());
        println!("  WCSPR: {:?}", router.wcspr());

        // Create and initialize trading pairs
        println!("\nCreating trading pairs...");
        env.set_gas(500_000_000_000);

        // Creating TokenA-TokenB pair
        let pair_a_b = factory.create_pair(token_a.address(), token_b.address());
        let pair_a_b_contract = PairHostRef::new(pair_a_b, env.clone());
        container.add_contract_named(&pair_a_b_contract, Some("TokenA_TokenB".to_string()))?;
        println!("  ✓ TokenA-TokenB pair created and initialized");

        // Create TokenA-WCSPR pair
        let pair_a_wcspr = factory.create_pair(token_a.address(), wcspr.address());
        let pair_a_wcspr_contract = PairHostRef::new(pair_a_wcspr, env.clone());
        println!("  ✓ TokenA-WCSPR pair created and initialized");
        container.add_contract_named(&pair_a_wcspr_contract, Some("TokenA_WCSPR".to_string()))?;

        // Create TokenB-WCSPR pair
        let pair_b_wcspr = factory.create_pair(token_b.address(), wcspr.address());
        let pair_b_wcspr_contract = PairHostRef::new(pair_b_wcspr, env.clone());
        println!("  ✓ TokenB-WCSPR pair created and initialized");
        container.add_contract_named(&pair_b_wcspr_contract, Some("TokenB_WCSPR".to_string()))?;

        let balance_after = env.balance_of(&env.caller());

        println!("\n✓ Deployment completed successfully!");
        println!("\nDeployed contracts:");
        println!("  - Factory - {:?}", factory.address());
        println!("  - SampleTokenA (TKNA) - {:?}", token_b.address());
        println!("  - SampleTokenB (TKNB) - {:?}", token_b.address());
        println!("  - WrappedNativeToken (WCSPR) - {:?}", wcspr.address());
        println!("  - Router - {:?}", router.factory_address());
        println!("\nTrading pairs:");
        println!("  - TokenA_TokenB - {:?}", pair_a_b.address());
        println!("  - TokenA_WCSPR - {:?}", pair_a_wcspr.address());
        println!("  - TokenB_WCSPR - {:?}", pair_b_wcspr.address());
        println!("\nUsed gas - {:?}", balance_before - balance_after);
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
        .contract::<Router>()
        .contract::<WrappedNativeToken>()
        .contract::<PairFactory>()
        .named_contract::<SampleToken>("SampleTokenA".to_string())
        .named_contract::<SampleToken>("SampleTokenB".to_string())
        .named_contract::<Pair>("TokenA_TokenB".to_string())
        .named_contract::<Pair>("TokenA_WCSPR".to_string())
        .named_contract::<Pair>("TokenB_WCSPR".to_string())
        .scenario(MintTokens)
        .scenario(SetupPair)
        .scenario(AddLiquidity)
        .scenario(AddLiquidityCSPR)
        .scenario(SwapTokens)
        .build()
        .run();
}
