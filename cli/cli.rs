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
use odra_modules::dapp::OwnedDappRegistry;
use odra_modules::wrapped_native::WrappedNativeToken;

mod scenarios;
use scenarios::dapp_registry::{connect_pair_factory, load_or_deploy_registry, register};
use scenarios::{
    AddLiquidity, AddLiquidityCSPR, AddWCSPR, CheckDappRegistry, CreateDappPair, DeployWcspr,
    MigrateToDappRegistry, SetupPair, SetupSamples, SwapTokens, UpgradeFactory, UpgradePairs,
    UpgradeRouter,
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

        let mut dapp_registry = load_or_deploy_registry(env, container).map_err(|e| {
            odra_cli::deploy::Error::OdraError {
                message: format!("{:?}", e),
            }
        })?;

        let mut pair_factory = PairFactory::load_or_deploy_with_cfg(
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
                dapp_registry: dapp_registry.address(),
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
                dapp_registry: dapp_registry.address(),
            },
            InstallConfig::upgradable::<Router>(),
            container,
            cspr!(500),
        )?;

        println!("Router deployed successfully!");

        // The contracts join the dapp as they are; the PairFactory adds the pairs the Factory creates.
        connect_pair_factory(
            env,
            &mut dapp_registry,
            &mut pair_factory,
            factory.address(),
        );
        register(env, &mut dapp_registry, "Factory", factory.address(), false);
        register(env, &mut dapp_registry, "Router", router.address(), false);
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
    let mut cli = OdraCli::new();
    // Contracts deployed by the CreateDappPair scenario.
    for suffix in ["2", "3"] {
        cli = cli
            .named_contract::<SampleToken>(format!("DappTokenA{}", suffix))
            .named_contract::<SampleToken>(format!("DappTokenB{}", suffix))
            .named_contract::<Pair>(format!("DappPair{}", suffix));
    }
    cli.about("Casper Trade CLI - Automated Market Maker on Casper Network")
        .deploy(ContractsDeployScript)
        .contract::<Factory>()
        .contract::<Router>()
        .contract::<WrappedNativeToken>()
        .contract::<PairFactory>()
        .contract::<OwnedDappRegistry>()
        .named_contract::<SampleToken>("SampleTokenA".to_string())
        .named_contract::<SampleToken>("SampleTokenB".to_string())
        .named_contract::<Pair>("TokenA_TokenB".to_string())
        .named_contract::<Pair>("TokenA_WCSPR".to_string())
        .named_contract::<Pair>("TokenB_WCSPR".to_string())
        .named_contract::<SampleToken>("DappTokenA".to_string())
        .named_contract::<SampleToken>("DappTokenB".to_string())
        .named_contract::<Pair>("DappPair".to_string())
        .scenario(SetupPair)
        .scenario(AddLiquidity)
        .scenario(AddLiquidityCSPR)
        .scenario(SwapTokens)
        .scenario(UpgradePairs)
        .scenario(UpgradeFactory)
        .scenario(UpgradeRouter)
        .scenario(AddWCSPR)
        .scenario(DeployWcspr)
        .scenario(SetupSamples)
        .scenario(MigrateToDappRegistry)
        .scenario(CreateDappPair)
        .scenario(CheckDappRegistry)
        .build()
        .run();
}
