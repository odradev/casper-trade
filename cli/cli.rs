//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.

use casperswap_contracts::casperswap_v2_pair::{CasperswapV2Pair, CasperswapV2PairInitArgs};
use casperswap_contracts::sample_tokens::{SampleTokenA, SampleTokenB};
use odra::host::HostEnv;
use odra_cli::{deploy::DeployScript, DeployedContractsContainer, DeployerExt, OdraCli};

/// Deploys the `CasperswapV2Pair` and `Flapper` contracts.
pub struct ContractsDeployScript;

impl DeployScript for ContractsDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer,
    ) -> Result<(), odra_cli::deploy::Error> {
        // Create mock token addresses for demonstration
        let _token0 = env.caller(); // Mock token0 address
        let _token1 = env.caller(); // Mock token1 address
        let factory = env.caller(); // Mock factory address

        let _ = CasperswapV2Pair::load_or_deploy(
            &env,
            None,
            CasperswapV2PairInitArgs { factory },
            container,
            250_000_000_000, // Adjust gas limit as needed
        )?;

        Ok(())
    }
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("CLI tool for casperswap smart contract")
        .deploy(ContractsDeployScript)
        .contract::<CasperswapV2Pair>()
        .build()
        .run();
}
