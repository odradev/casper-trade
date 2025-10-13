//! CLI tool for CasperSwap smart contracts
//!
//! This CLI provides deployment and interaction capabilities for the CasperSwap DEX contracts.

use casperswap_contracts::casperswap_v2_pair::{CasperswapV2Pair, CasperswapV2PairInitArgs};
use casperswap_contracts::factory::{Factory, FactoryInitArgs};
use casperswap_contracts::sample_tokens::{
    SampleTokenA, SampleTokenAInitArgs, SampleTokenB, SampleTokenBInitArgs,
};
use odra::casper_types::U256;
use odra::host::{HostEnv, InstallConfig};
use odra::prelude::Addressable;
use odra_cli::{cspr, deploy::DeployScript, DeployedContractsContainer, DeployerExt, OdraCli};

mod scenarios;
use scenarios::{CreatePair, MintTokens};

/// Deploys all CasperSwap contracts
pub struct ContractsDeployScript;

impl DeployScript for ContractsDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer,
    ) -> Result<(), odra_cli::deploy::Error> {
        env.set_gas(50_000_000_000);

        // Deploy Factory contract
        let factory = Factory::load_or_deploy_with_cfg(
            env,
            None,
            FactoryInitArgs {
                fee_to: Some(env.get_account(0)), // Set deployer as fee collector
            },
            InstallConfig::upgradable::<Factory>(),
            container,
            cspr!(400),
        )?;

        println!("Factory deployed successfully!");

        // Deploy Sample Token A
        let token_a = SampleTokenA::load_or_deploy_with_cfg(
            env,
            Some("SampleTokenA".to_string()),
            SampleTokenAInitArgs {
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
        let token_b = SampleTokenB::load_or_deploy_with_cfg(
            env,
            Some("SampleTokenB".to_string()),
            SampleTokenBInitArgs {
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

        // Deploy a CasperswapV2Pair contract (can be used as template)
        let _pair = CasperswapV2Pair::load_or_deploy_with_cfg(
            env,
            None,
            CasperswapV2PairInitArgs {
                factory: factory.address(),
            },
            InstallConfig::upgradable::<CasperswapV2Pair>(),
            container,
            cspr!(500),
        )?;

        
        println!("\nDeployment completed successfully!");
        println!("Use 'create-pair' scenario to create trading pairs.");

        Ok(())
    }
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("CasperSwap CLI - Automated Market Maker on Casper Network")
        .deploy(ContractsDeployScript)
        .contract::<Factory>()
        .contract::<CasperswapV2Pair>()
        .named_contract::<SampleTokenA>("SampleTokenA".to_string())
        .named_contract::<SampleTokenB>("SampleTokenB".to_string())
        .scenario(CreatePair)
        .scenario(MintTokens)
        .build()
        .run();
}
