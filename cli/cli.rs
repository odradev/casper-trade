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
use scenarios::dapp_registry::{point_factory_to_registry, register, set_dapp_metadata};
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

        // Deploy Router; it is also the dapp registry, owned by the deployer. Upgrading it took 469
        // CSPR on casper-test; the install is not measured.
        let mut router = Router::load_or_deploy_with_cfg(
            env,
            None,
            RouterInitArgs {
                factory: factory.address(),
                wcspr: wcspr_address,
            },
            InstallConfig::upgradable::<Router>(),
            container,
            cspr!(650),
        )?;

        println!("Router deployed successfully!");

        // The Factory joins the dapp as a registrar: it passes the registry to the pairs it creates
        // and adds them to the dapp.
        set_dapp_metadata(env, &mut router);
        point_factory_to_registry(env, &mut factory, router.address());
        register(env, &mut router, "Factory", factory.address(), true);
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
