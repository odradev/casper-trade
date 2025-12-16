//! CLI tool for Casper Trade smart contracts
//!
//! This CLI provides deployment and interaction capabilities for the Casper Trade DEX contracts.

use casper_trade_contracts::factory::{Factory, FactoryInitArgs};
use casper_trade_contracts::pair::{Pair, PairFactory};
use casper_trade_contracts::router::{Router, RouterInitArgs};
use casper_trade_contracts::sample_tokens::SampleToken;
use odra::host::{HostEnv, InstallConfig, NoArgs};
use odra::prelude::Addressable;
use odra_cli::{
    cspr, deploy::DeployScript, ContractProvider, DeployedContractsContainer, DeployerExt, OdraCli,
};
use odra_modules::wrapped_native::WrappedNativeToken;

mod scenarios;
use scenarios::{
    AddLiquidity, AddLiquidityCSPR, AddWCSPR, DeployWcspr, SetupPair, SetupSamples, SwapTokens,
    UpgradeFactory, UpgradePairs,
};

/// Deploys all Casper Trade contracts
pub struct ContractsDeployScript;

impl DeployScript for ContractsDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer,
    ) -> Result<(), odra_cli::deploy::Error> {
        let balance_before = env.balance_of(&env.caller());

        let wcspr_address = container
            .address_by_name("WrappedNativeToken")
            .expect("WCSPR not found in container! Run 'deploy-wcspr' or 'add-wcspr' first.");

        let pair_factory = PairFactory::load_or_deploy_with_cfg(
            env,
            None,
            NoArgs,
            InstallConfig::upgradable::<PairFactory>(),
            container,
            cspr!(799),
        )?;

        env.set_gas(300_000_000_000);
        // Deploy Factory contract
        let factory = Factory::load_or_deploy_with_cfg(
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

        // Deploy Router
        let router = Router::load_or_deploy_with_cfg(
            env,
            None,
            RouterInitArgs {
                factory: factory.address(),
                wcspr: wcspr_address,
            },
            InstallConfig::upgradable::<Router>(),
            container,
            cspr!(500),
        )?;

        println!("Router deployed successfully!");
        println!("  Factory: {:?}", router.factory_address());
        println!("  WCSPR: {:?}", router.wcspr());

        let balance_after = env.balance_of(&env.caller());

        println!("\n✓ Deployment completed successfully!");
        println!("\nDeployed contracts:");
        println!("  - Factory - {:?}", factory.address());
        println!("  - Router - {:?}", router.factory_address());
        println!("\nUsed gas - {:?}", balance_before - balance_after);
        println!("\nNext steps:");
        println!("  1. Run 'SetupSamples' scenario if you want to deploy sample tokens and pairs.");

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
        .scenario(SetupPair)
        .scenario(AddLiquidity)
        .scenario(AddLiquidityCSPR)
        .scenario(SwapTokens)
        .scenario(UpgradePairs)
        .scenario(UpgradeFactory)
        .scenario(AddWCSPR)
        .scenario(DeployWcspr)
        .scenario(SetupSamples)
        .build()
        .run();
}
