use super::utils::{create_token_ref, parse_token_input};
use casper_trade_contracts::pair::PairFactory;
use casper_trade_contracts::utils::contract_symbol;
use odra::host::HostEnv;
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};

pub struct UpgradePairs;

impl ScenarioMetadata for UpgradePairs {
    const NAME: &'static str = "UpgradePairs";
    const DESCRIPTION: &'static str = "Upgrade pairs created by PairFactory to the newest version";
}

impl Scenario for UpgradePairs {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new(
                "token_pairs",
                "Comma-separated list of token pairs to upgrade. Each pair should be in format 'tokenA:tokenB' where tokens can be contract names (e.g., SampleTokenA) or addresses (e.g., account-hash-...)\nExample: SampleTokenA:SampleTokenB,SampleTokenA:WrappedNativeToken",
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
        env.set_gas(1000_000_000_000);

        // Get token pairs from args
        let token_pairs_input = args.get_single::<String>("token_pairs")?;
        let pair_specs: Vec<&str> = token_pairs_input.split(',').map(|s| s.trim()).collect();

        if pair_specs.is_empty() {
            odra_cli::log("ERROR: At least one token pair must be provided");
            return Err(Error::OdraError {
                message: "At least one token pair must be provided".to_string(),
            });
        }

        // Get the PairFactory
        let mut pair_factory = container.contract_ref::<PairFactory>(env)?;

        odra_cli::log("Upgrading pairs created by PairFactory:");
        odra_cli::log(format!("  Token pairs to upgrade: {:?}", pair_specs));
        odra_cli::log("");

        // Process each pair
        for pair_spec in &pair_specs {
            // Parse the pair specification (tokenA:tokenB)
            let tokens: Vec<&str> = pair_spec.split(':').collect();
            if tokens.len() != 2 {
                odra_cli::log(format!(
                    "ERROR: Invalid pair specification '{}'. Expected format: 'tokenA:tokenB'",
                    pair_spec
                ));
                return Err(Error::OdraError {
                    message: format!(
                        "Invalid pair specification '{}'. Expected format: 'tokenA:tokenB'",
                        pair_spec
                    ),
                });
            }

            let token_a_input = tokens[0].trim();
            let token_b_input = tokens[1].trim();

            // Parse tokens using utility function
            let (token_a_address, token_a_display) =
                parse_token_input(token_a_input, "token_a", env, container)?;
            let (token_b_address, token_b_display) =
                parse_token_input(token_b_input, "token_b", env, container)?;

            // Create token instances to get names
            let token_a = create_token_ref(token_a_address, env);
            let token_b = create_token_ref(token_b_address, env);

            odra_cli::log(format!("Processing pair:"));
            odra_cli::log(format!(
                "  Token A: {} - {}",
                token_a_display,
                token_a.name()
            ));
            odra_cli::log(format!(
                "  Token B: {} - {}",
                token_b_display,
                token_b.name()
            ));

            // Sort tokens the same way the factory does
            let (token0_address, token1_address) = if token_a_address < token_b_address {
                (token_a_address, token_b_address)
            } else {
                (token_b_address, token_a_address)
            };

            let token_a = create_token_ref(token0_address, env);
            let token_b = create_token_ref(token1_address, env);

            let token_a_symbol = token_a.symbol();
            let token_b_symbol = token_b.symbol();

            let pair_symbol = contract_symbol(&token_a_symbol, &token_b_symbol);

            odra_cli::log(format!(
                "  Sorted tokens: token0={}, token1={}",
                token_a_symbol, token_b_symbol
            ));
            odra_cli::log(format!("  Pair contract name: {}", pair_symbol));
            odra_cli::log(format!("  Upgrading..."));

            // Use upgrade_child_contract method from the factory
            // The Pair::upgrade() method takes no arguments, so we just pass the contract name
            pair_factory.upgrade_child_contract(pair_symbol.clone());

            odra_cli::log(format!("  ✓ Pair {} upgraded successfully\n", pair_symbol));
        }

        odra_cli::log("✓ All pairs upgraded successfully!");

        Ok(())
    }
}
