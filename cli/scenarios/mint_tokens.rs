use casper_trade_contracts::sample_tokens::SampleToken;
use odra::casper_types::U256;
use odra::host::HostEnv;
use odra::prelude::Address;
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};

pub struct MintTokens;

impl ScenarioMetadata for MintTokens {
    const NAME: &'static str = "MintTokens";
    const DESCRIPTION: &'static str =
        "Mint tokens to a specified address (requires owner privileges)";
}

impl Scenario for MintTokens {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new("recipient", "Address of the recipient", NamedCLType::String)
                .required(),
            CommandArg::new(
                "amount",
                "Amount of tokens to mint (in base units, will be multiplied by 10^18)",
                NamedCLType::U64,
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

        // Get the token contract
        let mut token =
            container.contract_ref::<SampleToken>(env, Some("SampleTokenA".to_string()))?;

        // Get args
        let recipient_str = args.get_single::<String>("recipient")?;
        let amount_base = args.get_single::<u64>("amount")?;

        // Parse recipient address
        let recipient = recipient_str
            .parse::<Address>()
            .map_err(|_| Error::OdraError {
                message: "Invalid recipient address format".to_string(),
            })?;

        // Get token decimals
        let decimals = token.decimals();

        // Convert to token's smallest unit (using token's decimals)
        let amount = U256::from(amount_base) * U256::exp10(decimals as usize);

        odra_cli::log("Minting tokens:");
        odra_cli::log(format!("  Token: {}", token.symbol()));
        odra_cli::log(format!("  Recipient: {:?}", recipient));
        odra_cli::log(format!("  Amount: {} tokens", amount_base));

        // Get balance before
        let balance_before = token.balance_of(&recipient);

        // Mint tokens
        token.mint(&recipient, &amount);

        // Get balance after
        let balance_after = token.balance_of(&recipient);

        odra_cli::log("✓ Tokens minted successfully!");
        odra_cli::log(format!("  Balance before: {}", balance_before));
        odra_cli::log(format!("  Balance after: {}", balance_after));
        odra_cli::log(format!(
            "  Tokens added: {} tokens",
            (balance_after - balance_before) / U256::exp10(decimals as usize)
        ));

        Ok(())
    }
}
