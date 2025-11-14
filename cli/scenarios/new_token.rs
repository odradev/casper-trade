use casper_trade_contracts::sample_tokens::{SampleToken, SampleTokenInitArgs};
use odra::host::{Deployer, InstallConfig};
use odra::prelude::Addressable;
use odra::schema::casper_contract_schema::NamedCLType;
use odra::{casper_types::U256, host::HostEnv};
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, DeployedContractsContainer,
};

pub struct NewToken;

impl ScenarioMetadata for NewToken {
    const NAME: &'static str = "NewToken";
    const DESCRIPTION: &'static str = "Deploy a new SampleToken instance.";
}

impl Scenario for NewToken {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new("name", "Token name", NamedCLType::String).required(),
            CommandArg::new("symbol", "Token symbol", NamedCLType::String).required(),
            CommandArg::new("decimals", "Decimals", NamedCLType::U8).required(),
            CommandArg::new("initial_supply", "Initial supply", NamedCLType::U256).required(),
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer,
        args: Args,
    ) -> Result<(), Error> {
        env.set_gas(500_000_000_000);
        let name = args.get_single::<String>("name")?;
        let symbol = args.get_single::<String>("symbol")?;
        let decimals = args.get_single::<u8>("decimals")?;
        let initial_supply = args.get_single::<U256>("initial_supply")?;

        // Deploy the token
        let token = SampleToken::deploy_with_cfg(
            env,
            SampleTokenInitArgs {
                name: name.clone(),
                symbol: symbol.clone(),
                decimals,
                initial_supply,
            },
            InstallConfig::upgradable::<SampleToken>(),
        );

        let address = token.address();

        // Add the contract to the container with the symbol as the contract name
        container.add_contract_named(&token, Some(symbol.clone()))?;

        println!("\n✓ Token deployed successfully!");
        println!("  Name: {}", token.name());
        println!("  Symbol: {}", token.symbol());
        println!("  Decimals: {}", token.decimals());
        println!("  Total Supply: {}", token.total_supply());
        println!("  Address: {:?}", address);
        println!("\nToken added to container with name: {}", symbol);
        println!("You can now reference it by name: --token_a {}", symbol);
        println!(
            "Or by address: --token_a {:?}",
            address.to_formatted_string()
        );
        Ok(())
    }
}
