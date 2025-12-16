use casper_trade_contracts::pair::PairHostRef;
use casper_trade_contracts::sample_tokens::{SampleToken, SampleTokenInitArgs};
use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, HostRef, InstallConfig};
use odra::prelude::Addressable;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};

pub struct SetupSamples;

impl ScenarioMetadata for SetupSamples {
    const NAME: &'static str = "SetupSamples";
    const DESCRIPTION: &'static str = "Deploys sample tokens and creates testing pairs";
}

impl Scenario for SetupSamples {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args,
    ) -> Result<(), Error> {
        let factory_address = container
            .address_by_name("Factory")
            .expect("Factory not found in container");
        let mut factory =
            casper_trade_contracts::factory::FactoryHostRef::new(factory_address, env.clone());

        let wcspr_address = container
            .address_by_name("WrappedNativeToken")
            .expect("WCSPR not found in container");

        env.set_gas(500_000_000_000);
        odra_cli::log("Deploying Sample Token A...");
        let token_a = SampleToken::deploy_with_cfg(
            env,
            SampleTokenInitArgs {
                name: "Sample Token A".to_string(),
                symbol: "TKNA".to_string(),
                decimals: 18,
                initial_supply: U256::from(1_000_000_000u64) * U256::exp10(18),
            },
            InstallConfig {
                package_named_key: "sample_token_a".to_string(),
                is_upgradable: true,
                allow_key_override: true,
            },
        );
        container.add_contract_named(&token_a, Some("SampleTokenA".to_string()))?;
        odra_cli::log(format!(
            "✓ Sample Token A deployed: {:?}",
            token_a.address()
        ));

        odra_cli::log("Deploying Sample Token B...");
        let token_b = SampleToken::deploy_with_cfg(
            env,
            SampleTokenInitArgs {
                name: "Sample Token B".to_string(),
                symbol: "TKNB".to_string(),
                decimals: 18,
                initial_supply: U256::from(1_000_000_000u64) * U256::exp10(18),
            },
            InstallConfig {
                package_named_key: "sample_token_b".to_string(),
                is_upgradable: true,
                allow_key_override: true,
            },
        );
        container.add_contract_named(&token_b, Some("SampleTokenB".to_string()))?;
        odra_cli::log(format!(
            "✓ Sample Token B deployed: {:?}",
            token_b.address()
        ));

        odra_cli::log("\nCreating trading pairs...");
        env.set_gas(500_000_000_000);

        // TokenA-TokenB
        let pair_a_b = factory.create_pair(token_a.address(), token_b.address());
        let pair_a_b_contract = PairHostRef::new(pair_a_b, env.clone());
        container.add_contract_named(&pair_a_b_contract, Some("TokenA_TokenB".to_string()))?;
        odra_cli::log("✓ TokenA-TokenB pair created");

        // TokenA-WCSPR
        let pair_a_wcspr = factory.create_pair(token_a.address(), wcspr_address);
        let pair_a_wcspr_contract = PairHostRef::new(pair_a_wcspr, env.clone());
        container.add_contract_named(&pair_a_wcspr_contract, Some("TokenA_WCSPR".to_string()))?;
        odra_cli::log("✓ TokenA-WCSPR pair created");

        // TokenB-WCSPR
        let pair_b_wcspr = factory.create_pair(token_b.address(), wcspr_address);
        let pair_b_wcspr_contract = PairHostRef::new(pair_b_wcspr, env.clone());
        container.add_contract_named(&pair_b_wcspr_contract, Some("TokenB_WCSPR".to_string()))?;
        odra_cli::log("✓ TokenB-WCSPR pair created");

        Ok(())
    }
}
