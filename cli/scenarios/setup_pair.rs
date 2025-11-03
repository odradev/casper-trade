use super::utils::{create_token_ref, parse_token_input};
use casper_trade_contracts::casper_trade_v2_pair::CasperTradeV2Pair;
use casper_trade_contracts::factory::Factory;
use odra::host::HostEnv;
use odra::prelude::Addressable;
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};

pub struct SetupPair;

impl ScenarioMetadata for SetupPair {
    const NAME: &'static str = "SetupPair";
    const DESCRIPTION: &'static str =
        "Create, initialize, and register a trading pair for two tokens with the factory";
}

impl Scenario for SetupPair {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new(
                "token_a",
                "Contract name or address of the first token (e.g., SampleTokenA or account-hash-...)",
                NamedCLType::String,
            )
            .required(),
            CommandArg::new(
                "token_b",
                "Contract name or address of the second token (e.g., SampleTokenB or account-hash-...)",
                NamedCLType::String,
            )
            .required(),
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args,
    ) -> Result<(), Error> {
        env.set_gas(50_000_000_000);

        // Get args
        let token_a_input = args.get_single::<String>("token_a")?;
        let token_b_input = args.get_single::<String>("token_b")?;

        // Parse tokens using utility function
        let (token_a_address, token_a_name) =
            parse_token_input(&token_a_input, "token_a", env, container)?;
        let (token_b_address, token_b_name) =
            parse_token_input(&token_b_input, "token_b", env, container)?;

        // Create token instances
        let token_a = create_token_ref(token_a_address, env);
        let token_b = create_token_ref(token_b_address, env);

        // Get factory and pair
        let mut factory = container.contract_ref::<Factory>(env)?;
        let mut pair = container.contract_ref::<CasperTradeV2Pair>(env)?;

        odra_cli::log("Setting up trading pair:");
        odra_cli::log(format!(
            "  Token A: {} ({})",
            token_a.symbol(),
            token_a_name
        ));
        odra_cli::log(format!(
            "  Token B: {} ({})",
            token_b.symbol(),
            token_b_name
        ));
        odra_cli::log(format!("  Token A Address: {:?}", token_a_address));
        odra_cli::log(format!("  Token B Address: {:?}", token_b_address));
        odra_cli::log(format!("  Pair Address: {:?}", pair.address()));

        // Initialize the pair with the token addresses
        odra_cli::log("\nInitializing pair with token addresses...");
        pair.initialize(token_a_address, token_b_address);

        odra_cli::log(format!("  Token0: {:?}", pair.token0()));
        odra_cli::log(format!("  Token1: {:?}", pair.token1()));

        odra_cli::log("\n✓ Trading pair setup completed successfully!");
        odra_cli::log("  You can now add liquidity using the 'AddLiquidity' scenario");

        Ok(())
    }
}
