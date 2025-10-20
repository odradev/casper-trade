use super::utils::{create_token_ref, parse_token_input};
use casperswap_contracts::router::CasperswapV2Router;
use odra::casper_types::U256;
use odra::host::HostEnv;
use odra::prelude::Addressable;
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};

pub struct AddLiquidity;

impl ScenarioMetadata for AddLiquidity {
    const NAME: &'static str = "AddLiquidity";
    const DESCRIPTION: &'static str = "Add liquidity to a token pair using the router";
}

impl Scenario for AddLiquidity {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new("token_a", "Contract name or address of the first token (e.g., SampleTokenA or account-hash-...)", NamedCLType::String)
                .required(),
            CommandArg::new("token_b", "Contract name or address of the second token (e.g., SampleTokenB or account-hash-...)", NamedCLType::String)
                .required(),
            CommandArg::new(
                "amount_a",
                "Amount of token A to add (in whole tokens, will be multiplied by 10^18)",
                NamedCLType::U64,
            )
            .required(),
            CommandArg::new(
                "amount_b",
                "Amount of token B to add (in whole tokens, will be multiplied by 10^18)",
                NamedCLType::U64,
            )
            .required(),
            CommandArg::new(
                "slippage",
                "Slippage tolerance in percentage (default: 1%)",
                NamedCLType::U64,
            ),
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args,
    ) -> Result<(), Error> {
        env.set_gas(50_000_000_000);

        // Get the router contract
        let mut router = container.contract_ref::<CasperswapV2Router>(env, None)?;

        // Get args
        let token_a_input = args.get_single::<String>("token_a")?;
        let token_b_input = args.get_single::<String>("token_b")?;
        let amount_a_base = args.get_single::<u64>("amount_a")?;
        let amount_b_base = args.get_single::<u64>("amount_b")?;
        let slippage = args.get_single::<u64>("slippage").unwrap_or(1);

        // Parse tokens using utility function
        let (token_a_address, token_a_name) =
            parse_token_input(&token_a_input, "token_a", env, container)?;
        let (token_b_address, token_b_name) =
            parse_token_input(&token_b_input, "token_b", env, container)?;

        // Create token instances
        let mut token_a = create_token_ref(token_a_address, env);
        let mut token_b = create_token_ref(token_b_address, env);

        // Get token decimals
        let decimals_a = token_a.decimals();
        let decimals_b = token_b.decimals();

        // Convert to token's smallest unit (using token's decimals)
        let amount_a = U256::from(amount_a_base) * U256::exp10(decimals_a as usize);
        let amount_b = U256::from(amount_b_base) * U256::exp10(decimals_b as usize);

        // Calculate minimum amounts with slippage tolerance
        let amount_a_min = amount_a * U256::from(100 - slippage) / U256::from(100);
        let amount_b_min = amount_b * U256::from(100 - slippage) / U256::from(100);

        let caller = env.caller();

        odra_cli::log("Adding liquidity:");
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
        odra_cli::log(format!("  Amount A: {} tokens", amount_a_base));
        odra_cli::log(format!("  Amount B: {} tokens", amount_b_base));
        odra_cli::log(format!("  Slippage tolerance: {}%", slippage));
        odra_cli::log(format!("  Caller: {:?}", caller));

        // Check balances
        let balance_a = token_a.balance_of(&caller);
        let balance_b = token_b.balance_of(&caller);
        odra_cli::log("\nBalances before:");
        odra_cli::log(format!(
            "  {}: {}",
            token_a.symbol(),
            balance_a / U256::exp10(decimals_a as usize)
        ));
        odra_cli::log(format!(
            "  {}: {}",
            token_b.symbol(),
            balance_b / U256::exp10(decimals_b as usize)
        ));

        if balance_a < amount_a {
            return Err(Error::OdraError {
                message: format!(
                    "Insufficient {} balance. Have: {}, Need: {}",
                    token_a.symbol(),
                    balance_a / U256::exp10(decimals_a as usize),
                    amount_a_base
                ),
            });
        }

        if balance_b < amount_b {
            return Err(Error::OdraError {
                message: format!(
                    "Insufficient {} balance. Have: {}, Need: {}",
                    token_b.symbol(),
                    balance_b / U256::exp10(decimals_b as usize),
                    amount_b_base
                ),
            });
        }

        // Approve router to spend tokens
        odra_cli::log("\nApproving router to spend tokens...");
        token_a.approve(&router.address(), &amount_a);
        token_b.approve(&router.address(), &amount_b);

        // Add liquidity
        odra_cli::log("Adding liquidity to pair...");
        let (amount_a_used, amount_b_used, liquidity) = router.add_liquidity(
            token_a_address,
            token_b_address,
            amount_a,
            amount_b,
            amount_a_min,
            amount_b_min,
            caller,
            u64::MAX, // deadline
        );

        odra_cli::log("\n✓ Liquidity added successfully!");
        odra_cli::log(format!(
            "  {} used: {} tokens",
            token_a.symbol(),
            amount_a_used / U256::exp10(decimals_a as usize)
        ));
        odra_cli::log(format!(
            "  {} used: {} tokens",
            token_b.symbol(),
            amount_b_used / U256::exp10(decimals_b as usize)
        ));
        odra_cli::log(format!(
            "  LP tokens received: {}",
            liquidity / U256::exp10(18)
        ));

        // Check balances after
        let balance_a_after = token_a.balance_of(&caller);
        let balance_b_after = token_b.balance_of(&caller);
        odra_cli::log("\nBalances after:");
        odra_cli::log(format!(
            "  {}: {}",
            token_a.symbol(),
            balance_a_after / U256::exp10(decimals_a as usize)
        ));
        odra_cli::log(format!(
            "  {}: {}",
            token_b.symbol(),
            balance_b_after / U256::exp10(decimals_b as usize)
        ));

        Ok(())
    }
}
