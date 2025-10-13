use casperswap_contracts::casperswap_v2_pair::CasperswapV2Pair;
use odra::host::HostEnv;
use odra::prelude::Address;
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};

pub struct CreatePair;

impl ScenarioMetadata for CreatePair {
    const NAME: &'static str = "CreatePair";
    const DESCRIPTION: &'static str = "Initialize a CasperswapV2Pair with two token addresses";
}

impl Scenario for CreatePair {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new("token0", "Address of the first token", NamedCLType::String).required(),
            CommandArg::new("token1", "Address of the second token", NamedCLType::String)
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

        let mut pair = container.contract_ref::<CasperswapV2Pair>(env, None)?;

        // Get token addresses from args
        let token0_str = args.get_single::<String>("token0")?;
        let token1_str = args.get_single::<String>("token1")?;

        // Parse addresses
        let token0 = token0_str
            .parse::<Address>()
            .map_err(|_| Error::OdraError {
                message: "Invalid token0 address format".to_string(),
            })?;
        let token1 = token1_str
            .parse::<Address>()
            .map_err(|_| Error::OdraError {
                message: "Invalid token1 address format".to_string(),
            })?;

        // Ensure token0 < token1 (Uniswap V2 convention)
        let (token0, token1) = if format!("{:?}", token0) < format!("{:?}", token1) {
            (token0, token1)
        } else {
            (token1, token0)
        };

        odra_cli::log(format!("Initializing pair with:"));
        odra_cli::log(format!("  token0: {:?}", token0));
        odra_cli::log(format!("  token1: {:?}", token1));

        // Initialize the pair
        pair.initialize(token0, token1);

        odra_cli::log("✓ Pair initialized successfully!");

        Ok(())
    }
}
