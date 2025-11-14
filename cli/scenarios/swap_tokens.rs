use super::utils::{create_token_ref, parse_token_input};
use casper_trade_contracts::router::Router;
use odra::casper_types::U256;
use odra::host::HostEnv;
use odra::prelude::Addressable;
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};

pub struct SwapTokens;

impl ScenarioMetadata for SwapTokens {
    const NAME: &'static str = "SwapTokens";
    const DESCRIPTION: &'static str = "Swap tokens using the router";
}

impl Scenario for SwapTokens {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new(
                "token_in",
                "Contract name or address of the input token (e.g., SampleTokenA or account-hash-...)",
                NamedCLType::String,
            )
            .required(),
            CommandArg::new(
                "token_out",
                "Contract name or address of the output token (e.g., SampleTokenB or account-hash-...)",
                NamedCLType::String,
            )
            .required(),
            CommandArg::new(
                "amount_in",
                "Amount of input token to swap (in whole tokens, will be multiplied by 10^18)",
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
        container: &mut DeployedContractsContainer,
        args: Args,
    ) -> Result<(), Error> {
        env.set_gas(50_000_000_000);

        // Get the router contract
        let mut router = container.contract_ref::<Router>(env)?;

        // Get args
        let token_in_input = args.get_single::<String>("token_in")?;
        let token_out_input = args.get_single::<String>("token_out")?;
        let amount_in_base = args.get_single::<u64>("amount_in")?;
        let slippage = args.get_single::<u64>("slippage").unwrap_or(1);

        // Parse tokens using utility function
        let (token_in_address, token_in_name) =
            parse_token_input(&token_in_input, "token_in", env, container)?;
        let (token_out_address, token_out_name) =
            parse_token_input(&token_out_input, "token_out", env, container)?;

        // Create token instances
        let mut token_in = create_token_ref(token_in_address, env);
        let token_out = create_token_ref(token_out_address, env);

        let decimals = token_in.decimals();
        let amount_in = U256::from(amount_in_base) * U256::exp10(decimals as usize);

        let caller = env.caller();

        odra_cli::log("Swapping tokens:");
        odra_cli::log(format!(
            "  Input: {} ({}) - {} tokens",
            token_in.symbol(),
            token_in_name,
            amount_in_base
        ));
        odra_cli::log(format!(
            "  Output: {} ({})",
            token_out.symbol(),
            token_out_name
        ));
        odra_cli::log(format!("  Input Address: {:?}", token_in_address));
        odra_cli::log(format!("  Output Address: {:?}", token_out_address));
        odra_cli::log(format!("  Slippage tolerance: {}%", slippage));
        odra_cli::log(format!("  Caller: {:?}", caller));

        // Check balance
        let balance_in_before = token_in.balance_of(&caller);
        let balance_out_before = token_out.balance_of(&caller);
        let decimals_out = token_out.decimals();

        odra_cli::log("\nBalances before:");
        odra_cli::log(format!(
            "  {}: {} tokens",
            token_in.symbol(),
            balance_in_before / U256::exp10(decimals as usize)
        ));
        odra_cli::log(format!(
            "  {}: {} tokens",
            token_out.symbol(),
            balance_out_before / U256::exp10(decimals_out as usize)
        ));

        if balance_in_before < amount_in {
            return Err(Error::OdraError {
                message: format!(
                    "Insufficient {} balance. Have: {}, Need: {}",
                    token_in.symbol(),
                    balance_in_before / U256::exp10(decimals as usize),
                    amount_in_base
                ),
            });
        }

        // Get expected output amount
        let path = vec![token_in_address, token_out_address];
        let amounts_out = router.get_amounts_out(amount_in, path.clone());
        let expected_amount_out = amounts_out[1];

        odra_cli::log(format!(
            "\nExpected output: {} {} tokens",
            token_out.symbol(),
            expected_amount_out / U256::exp10(decimals_out as usize)
        ));

        // Calculate minimum output with slippage tolerance
        let amount_out_min = expected_amount_out * U256::from(100 - slippage) / U256::from(100);

        odra_cli::log(format!(
            "Minimum output (with {}% slippage): {} tokens",
            slippage,
            amount_out_min / U256::exp10(decimals_out as usize)
        ));

        // Approve router to spend input tokens
        odra_cli::log("\nApproving router to spend tokens...");
        token_in.approve(&router.address(), &amount_in);

        // Perform swap
        odra_cli::log("Executing swap...");
        let amounts = router.swap_exact_tokens_for_tokens(
            amount_in,
            amount_out_min,
            path,
            caller,
            u64::MAX, // deadline
        );

        let actual_amount_out = amounts[1];

        odra_cli::log("\n✓ Swap completed successfully!");
        odra_cli::log(format!(
            "  {} sent: {} tokens",
            token_in.symbol(),
            amounts[0] / U256::exp10(decimals as usize)
        ));
        odra_cli::log(format!(
            "  {} received: {} tokens",
            token_out.symbol(),
            actual_amount_out / U256::exp10(decimals_out as usize)
        ));

        // Check balances after
        let balance_in_after = token_in.balance_of(&caller);
        let balance_out_after = token_out.balance_of(&caller);

        odra_cli::log("\nBalances after:");
        odra_cli::log(format!(
            "  {}: {} tokens",
            token_in.symbol(),
            balance_in_after / U256::exp10(decimals as usize)
        ));
        odra_cli::log(format!(
            "  {}: {} tokens",
            token_out.symbol(),
            balance_out_after / U256::exp10(decimals_out as usize)
        ));

        // Calculate effective exchange rate (in human-readable format)
        // Normalize both amounts to the same decimal places for accurate rate calculation
        let amount_in_normalized = amount_in / U256::exp10(decimals as usize);
        let amount_out_normalized = actual_amount_out / U256::exp10(token_out.decimals() as usize);

        // Calculate rate with 4 decimal precision: (output / input) * 10000
        let rate = if amount_in_normalized > U256::zero() {
            (amount_out_normalized * U256::from(10000)) / amount_in_normalized
        } else {
            U256::zero()
        };

        // Display rate as decimal (e.g., 9444 means 0.9444)
        let rate_integer = rate / U256::from(10000);
        let rate_decimal = rate % U256::from(10000);
        odra_cli::log(format!(
            "\nEffective rate: 1 {} = {}.{:04} {} (with 0.3% fee)",
            token_in.symbol(),
            rate_integer,
            rate_decimal,
            token_out.symbol()
        ));

        Ok(())
    }
}
