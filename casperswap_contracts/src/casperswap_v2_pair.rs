use odra::{
    casper_types::{bytesrepr::Bytes, U256},
    prelude::*,
    ContractRef,
};
use odra_modules::cep18_token::{Cep18, Cep18ContractRef};

use crate::{
    casperswap_callee::CasperswapCalleeContractRef,
    casperswap_v2_pair::{
        errors::CasperswapV2PairError,
        events::{Mint, Swap, Sync},
    },
    factory::FactoryContractRef,
    utils::zero_address,
};
pub mod errors;
pub mod events;

pub const MINIMUM_LIQUIDITY: u64 = 1000;

/// CasperswapV2Pair contract - implementation based on Uniswap V2
#[odra::module(events = [Mint, Swap, Sync])]
pub struct CasperswapV2Pair {
    pub token: SubModule<Cep18>,
    pub factory: Var<Address>,
    pub token0: Var<Address>,
    pub token1: Var<Address>,
    pub reserve0: Var<U256>,
    pub reserve1: Var<U256>,
    pub k_last: Var<U256>,
    pub block_timestamp_last: Var<u64>,
    pub price0_cumulative_last: Var<U256>,
    pub price1_cumulative_last: Var<U256>,
}

/// Module implementation
#[odra::module]
impl CasperswapV2Pair {
    delegate! {
        to self.token {
            fn total_supply(&self) -> U256;
            fn balance_of(&self, address: &Address) -> U256;
            fn transfer(&mut self, recipient: &Address, amount: &U256);
            fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256);
            fn approve(&mut self, spender: &Address, amount: &U256);
            fn allowance(&self, owner: &Address, spender: &Address) -> U256;
        }
    }

    pub fn init(&mut self, factory: Address) {
        self.factory.set(factory);
        let symbol = "LP";
        let name = "CasperswapV2Pair";
        let decimals = 18;
        let initial_supply = U256::from(0);
        self.token.init(
            symbol.to_string(),
            name.to_string(),
            decimals,
            initial_supply,
        );
    }

    pub fn initialize(&mut self, token0: Address, token1: Address) {
        // TODO: Uncomment this when the factory is implemented
        // if self.factory.get_or_revert_with(CasperswapV2PairError::Misconfigured) != self.env().caller() {
        //     self.env().revert(CasperswapV2PairError::Forbidden);
        // }
        self.token0.set(token0);
        self.token1.set(token1);
    }

    #[odra(non_reentrant)]
    pub fn mint(&mut self, to: Address) {
        // TODO: below should be zero address or some kind of locking mechanism
        let zero_address = zero_address();
        let balance0 = self.token0().balance_of(&self.env().self_address());
        let balance1 = self.token1().balance_of(&self.env().self_address());
        let reserve0 = self.reserve0.get_or_default();
        let reserve1 = self.reserve1.get_or_default();

        let amount0 = balance0 - reserve0;
        let amount1 = balance1 - reserve1;

        let fee_on = self._mint_fee(reserve0, reserve1);
        let total_supply = self.total_supply();
        let minimum_liquidity = U256::from(MINIMUM_LIQUIDITY);
        let liquidity = if total_supply.is_zero() {
            // permanently lock the first MINIMUM_LIQUIDITY tokensp
            self.token.raw_mint(&zero_address, &minimum_liquidity);
            (amount0 * amount1).integer_sqrt() - minimum_liquidity
        } else {
            (amount0 * total_supply / reserve0).min(amount1 * total_supply / reserve1)
        };

        if liquidity.is_zero() {
            self.env()
                .revert(CasperswapV2PairError::InsufficientLiquidityMinted);
        }

        self.token.raw_mint(&to, &liquidity);
        self._update(balance0, balance1, reserve0, reserve1);

        if fee_on {
            self.k_last.set(reserve0 * reserve1);
        }

        self.env().emit_event(Mint {
            sender: self.env().caller(),
            amount0,
            amount1,
        });
    }

    #[odra(non_reentrant)]
    pub fn swap(&mut self, amount0_out: U256, amount1_out: U256, to: Address, data: Option<Bytes>) {
        // Require at least one output amount to be > 0
        if amount0_out.is_zero() && amount1_out.is_zero() {
            self.env()
                .revert(CasperswapV2PairError::InsufficientOutputAmount);
        }

        // Get reserves
        let reserve0 = self.reserve0.get_or_default();
        let reserve1 = self.reserve1.get_or_default();

        // Require outputs are less than reserves
        if amount0_out >= reserve0 || amount1_out >= reserve1 {
            self.env()
                .revert(CasperswapV2PairError::InsufficientLiquidity);
        }

        // Get token addresses
        let token0_addr = self
            .token0
            .get_or_revert_with(CasperswapV2PairError::NotInitialized);
        let token1_addr = self
            .token1
            .get_or_revert_with(CasperswapV2PairError::NotInitialized);

        // Validate recipient is not one of the token addresses
        if to == token0_addr || to == token1_addr {
            self.env().revert(CasperswapV2PairError::InvalidTo);
        }

        // Optimistically transfer tokens
        if !amount0_out.is_zero() {
            self.token0().transfer(&to, &amount0_out);
        }
        if !amount1_out.is_zero() {
            self.token1().transfer(&to, &amount1_out);
        }

        // Call the callback if data is provided
        if let Some(callback_data) = data {
            let callee = CasperswapCalleeContractRef::new(self.env(), to);
            callee.casperswap_call(self.env().caller(), amount0_out, amount1_out, callback_data);
        }

        // Get new balances
        let balance0 = self.token0().balance_of(&self.env().self_address());
        let balance1 = self.token1().balance_of(&self.env().self_address());

        // Calculate input amounts
        let amount0_in = if balance0 > reserve0 - amount0_out {
            balance0 - (reserve0 - amount0_out)
        } else {
            U256::zero()
        };

        let amount1_in = if balance1 > reserve1 - amount1_out {
            balance1 - (reserve1 - amount1_out)
        } else {
            U256::zero()
        };

        // Require at least one input amount > 0
        if amount0_in.is_zero() && amount1_in.is_zero() {
            self.env()
                .revert(CasperswapV2PairError::InsufficientInputAmount);
        }

        // Check K invariant with 0.3% fee
        // balance * 1000 - amount_in * 3 (0.3% fee)
        let balance0_adjusted = balance0 * U256::from(1000) - amount0_in * U256::from(3);
        let balance1_adjusted = balance1 * U256::from(1000) - amount1_in * U256::from(3);
        let k_adjusted = balance0_adjusted * balance1_adjusted;
        let k_original = reserve0 * reserve1 * U256::from(1000).pow(U256::from(2));

        if k_adjusted < k_original {
            self.env().revert(CasperswapV2PairError::K);
        }

        // Update reserves
        self._update(balance0, balance1, reserve0, reserve1);

        // Emit Swap event
        self.env().emit_event(Swap {
            sender: self.env().caller(),
            amount0_in,
            amount1_in,
            amount0_out,
            amount1_out,
            to,
        });
    }

    pub fn get_reserve0(&self) -> U256 {
        self.reserve0.get_or_default()
    }

    pub fn get_reserve1(&self) -> U256 {
        self.reserve1.get_or_default()
    }
}

impl CasperswapV2Pair {
    // TODO: Verify the soundness of this function
    fn _update(&mut self, balance0: U256, balance1: U256, reserve0: U256, reserve1: U256) {
        // Get current block timestamp
        let block_timestamp = self.env().get_block_time();
        let block_timestamp_last = self.block_timestamp_last.get_or_default();

        // Calculate time elapsed (overflow is desired, so we use wrapping_sub)
        let time_elapsed = block_timestamp.wrapping_sub(block_timestamp_last);

        // Update price accumulators if time has elapsed and reserves exist
        if time_elapsed > 0 && !reserve0.is_zero() && !reserve1.is_zero() {
            // Calculate price0 = reserve1 / reserve0 (encoded as UQ112x112)
            let price0 = self._uqdiv(reserve1, reserve0);
            let price0_cumulative_last = self.price0_cumulative_last.get_or_default();
            self.price0_cumulative_last
                .set(price0_cumulative_last + (price0 * U256::from(time_elapsed)));

            // Calculate price1 = reserve0 / reserve1 (encoded as UQ112x112)
            let price1 = self._uqdiv(reserve0, reserve1);
            let price1_cumulative_last = self.price1_cumulative_last.get_or_default();
            self.price1_cumulative_last
                .set(price1_cumulative_last + (price1 * U256::from(time_elapsed)));
        }

        // Update reserves
        self.reserve0.set(balance0);
        self.reserve1.set(balance1);
        self.block_timestamp_last.set(block_timestamp);

        // Emit Sync event
        self.env().emit_event(Sync {
            reserve0: balance0,
            reserve1: balance1,
        });
    }

    /// UQ112x112 division - equivalent to UQ112x112.encode(x).uqdiv(y)
    fn _uqdiv(&self, x: U256, y: U256) -> U256 {
        // UQ112x112 means we have 112 bits for integer part and 112 bits for fractional part
        // So we multiply by 2^112 to get the fixed-point representation
        let q112 = U256::from(2u64.pow(112));
        (x * q112) / y
    }

    // if fee is on, mint liquidity equivalent to 1/6th of the growth in sqrt(k)
    fn _mint_fee(&mut self, reserve0: U256, reserve1: U256) -> bool {
        let fee_to = self.factory().fee_to();
        let fee_on = fee_to.is_some();

        let k_last = self.k_last.get_or_default();

        if fee_on {
            if !k_last.is_zero() {
                let root_k = (reserve0 * reserve1).integer_sqrt();
                let root_k_last = k_last.integer_sqrt();
                if root_k > root_k_last {
                    let numerator = self.total_supply() * (root_k - root_k_last);
                    let denominator = root_k * 5 + root_k_last;
                    let liquidity = numerator / denominator;
                    if liquidity > U256::zero() {
                        self.token.raw_mint(&fee_to.unwrap(), &liquidity);
                    }
                }
            }
        } else if k_last != U256::zero() {
            self.k_last.set(U256::zero());
        }

        fee_on
    }

    fn factory(&self) -> FactoryContractRef {
        FactoryContractRef::new(
            self.env(),
            self.factory
                .get_or_revert_with(CasperswapV2PairError::NotInitialized),
        )
    }

    fn token0(&self) -> Cep18ContractRef {
        Cep18ContractRef::new(
            self.env(),
            self.token0
                .get_or_revert_with(CasperswapV2PairError::NotInitialized),
        )
    }

    fn token1(&self) -> Cep18ContractRef {
        Cep18ContractRef::new(
            self.env(),
            self.token1
                .get_or_revert_with(CasperswapV2PairError::NotInitialized),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        factory::{Factory, FactoryInitArgs},
        sample_tokens::{
            SampleTokenA, SampleTokenAHostRef, SampleTokenAInitArgs, SampleTokenB,
            SampleTokenBHostRef, SampleTokenBInitArgs,
        },
        utils::expand_to_18_decimals,
    };

    use super::*;
    use odra::{
        casper_types::U256,
        host::{Deployer, HostEnv},
    };

    struct PairEnv {
        pub odra_env: HostEnv,
        pub pair: CasperswapV2PairHostRef,
        pub token0: SampleTokenAHostRef,
        pub token1: SampleTokenBHostRef,
        pub owner: Address,
        pub alice: Address,
        pub bob: Address,
    }

    fn setup() -> PairEnv {
        let env = odra_test::env();
        let factory = Factory::deploy(
            &env,
            FactoryInitArgs {
                fee_to: Some(env.get_account(0)),
            },
        );
        let token0 = SampleTokenA::deploy(
            &env,
            SampleTokenAInitArgs {
                name: "Sample Token A".to_string(),
                symbol: "STA".to_string(),
                decimals: 18,
                initial_supply: expand_to_18_decimals(10000),
            },
        );
        let token1 = SampleTokenB::deploy(
            &env,
            SampleTokenBInitArgs {
                name: "Sample Token B".to_string(),
                symbol: "STB".to_string(),
                decimals: 18,
                initial_supply: expand_to_18_decimals(10000),
            },
        );
        let mut pair = CasperswapV2Pair::deploy(
            &env,
            CasperswapV2PairInitArgs {
                factory: factory.address(),
            },
        );
        pair.initialize(token0.address(), token1.address());
        let owner = env.get_account(0);
        let alice = env.get_account(1);
        let bob = env.get_account(2);
        PairEnv {
            odra_env: env,
            pair,
            token0,
            token1,
            owner,
            alice,
            bob,
        }
    }

    fn add_liquidity(env: &mut PairEnv, token0amount: U256, token1amount: U256) {
        env.token0.transfer(&env.pair.address(), &token0amount);
        env.token1.transfer(&env.pair.address(), &token1amount);
        env.pair.mint(env.alice);
    }

    #[test]
    fn mint() {
        let mut env = setup();
        let token0amount = expand_to_18_decimals(1);
        let token1amount = expand_to_18_decimals(4);

        env.token0.transfer(&env.pair.address(), &token0amount);
        env.token1.transfer(&env.pair.address(), &token1amount);

        let expected_liquidity = expand_to_18_decimals(2);

        env.pair.mint(env.alice);

        assert_eq!(env.pair.total_supply(), expected_liquidity);
        assert_eq!(
            env.pair.balance_of(&env.alice),
            expected_liquidity.saturating_sub(U256::from(MINIMUM_LIQUIDITY))
        );
        assert_eq!(
            env.token0.balance_of(&env.pair.address()),
            U256::from(token0amount)
        );
        assert_eq!(
            env.token1.balance_of(&env.pair.address()),
            U256::from(token1amount)
        );

        let reserve0 = env.pair.get_reserve0();
        let reserve1 = env.pair.get_reserve1();
        assert_eq!(reserve0, U256::from(token0amount));
        assert_eq!(reserve1, U256::from(token1amount));
    }

    #[test]
    fn swap_token0() {
        let mut env = setup();
        let token0amount = expand_to_18_decimals(5);
        let token1amount = expand_to_18_decimals(10);

        add_liquidity(&mut env, token0amount, token1amount);

        let swap_amount = expand_to_18_decimals(1);
        let expected_output_amount = U256::from(1662497915624478906 as u128);

        env.token0.transfer(&env.pair.address(), &swap_amount);
        env.pair
            .swap(U256::zero(), expected_output_amount, env.owner, None);

        let reserve0 = env.pair.get_reserve0();
        let reserve1 = env.pair.get_reserve1();

        assert_eq!(reserve0, token0amount + swap_amount);
        assert_eq!(reserve1, token1amount - expected_output_amount);

        assert_eq!(
            env.token0.balance_of(&env.pair.address()),
            token0amount + swap_amount
        );
        assert_eq!(
            env.token1.balance_of(&env.pair.address()),
            token1amount - expected_output_amount
        );

        let total_supply_token0 = env.token0.total_supply();
        let total_supply_token1 = env.token1.total_supply();

        assert_eq!(
            env.token0.balance_of(&env.owner),
            total_supply_token0 - token0amount - swap_amount
        );
        assert_eq!(
            env.token1.balance_of(&env.owner),
            total_supply_token1 - token1amount + expected_output_amount
        );
    }

    #[test]
    fn swap_token1() {
        let mut env = setup();
        let token0amount = expand_to_18_decimals(5);
        let token1amount = expand_to_18_decimals(10);

        add_liquidity(&mut env, token0amount, token1amount);

        let swap_amount = expand_to_18_decimals(1);
        let expected_output_amount = U256::from(453305446940074565 as u128);

        env.token1.transfer(&env.pair.address(), &swap_amount);
        env.pair
            .swap(expected_output_amount, U256::zero(), env.owner, None);

        let reserve0 = env.pair.get_reserve0();
        let reserve1 = env.pair.get_reserve1();

        assert_eq!(reserve0, token0amount - expected_output_amount);
        assert_eq!(reserve1, token1amount + swap_amount);

        assert_eq!(
            env.token0.balance_of(&env.pair.address()),
            token0amount - expected_output_amount
        );
        assert_eq!(
            env.token1.balance_of(&env.pair.address()),
            token1amount + swap_amount
        );

        let total_supply_token0 = env.token0.total_supply();
        let total_supply_token1 = env.token1.total_supply();

        assert_eq!(
            env.token0.balance_of(&env.owner),
            total_supply_token0 - token0amount + expected_output_amount
        );
        assert_eq!(
            env.token1.balance_of(&env.owner),
            total_supply_token1 - token1amount - swap_amount
        );
    }

    #[test]
    fn swap_test_cases() {
        // Test cases: [swapAmount, token0Amount, token1Amount, expectedOutputAmount]
        let swap_test_cases: Vec<(u64, u64, u64, u128)> = vec![
            (1, 5, 10, 1662497915624478906),
            (1, 10, 5, 453305446940074565),
            (2, 5, 10, 2851015155847869602),
            (2, 10, 5, 831248957812239453),
            (1, 10, 10, 906610893880149131),
            (1, 100, 100, 987158034397061298),
            (1, 1000, 1000, 996006981039903216),
        ];

        for (i, (swap_amount, token0_amount, token1_amount, expected_output)) in
            swap_test_cases.iter().enumerate()
        {
            let mut env = setup();
            let swap_amount = expand_to_18_decimals(*swap_amount);
            let token0amount = expand_to_18_decimals(*token0_amount);
            let token1amount = expand_to_18_decimals(*token1_amount);
            let expected_output_amount = U256::from(*expected_output);

            add_liquidity(&mut env, token0amount, token1amount);
            env.token0.transfer(&env.pair.address(), &swap_amount);

            // Try with expectedOutputAmount + 1, should fail
            let result = env.pair.try_swap(
                U256::zero(),
                expected_output_amount + U256::one(),
                env.owner,
                None,
            );
            assert!(result.is_err(), "Test case {} should fail with K error", i);

            // Should succeed with exact expectedOutputAmount
            env.pair
                .swap(U256::zero(), expected_output_amount, env.owner, None);
        }
    }

    #[test]
    fn optimistic() {
        // Test cases: [outputAmount, token0Amount, token1Amount, inputAmount]
        // First 3 cases: given amountIn, amountOut = floor(amountIn * .997)
        // Last case: given amountOut, amountIn = ceiling(amountOut / .997)
        let optimistic_test_cases: Vec<(Option<u64>, u64, u64, Option<u64>, u128)> = vec![
            (None, 5, 10, Some(1), 997000000000000000),
            (None, 10, 5, Some(1), 997000000000000000),
            (None, 5, 5, Some(1), 997000000000000000),
            (Some(1), 5, 5, None, 1003009027081243732),
        ];

        for (i, (output_18, token0_amount, token1_amount, input_18, amount_val)) in
            optimistic_test_cases.iter().enumerate()
        {
            let mut env = setup();
            let token0amount = expand_to_18_decimals(*token0_amount);
            let token1amount = expand_to_18_decimals(*token1_amount);

            let (output_amount, input_amount) = if let Some(output) = output_18 {
                (expand_to_18_decimals(*output), U256::from(*amount_val))
            } else {
                (U256::from(*amount_val), expand_to_18_decimals(input_18.unwrap()))
            };

            add_liquidity(&mut env, token0amount, token1amount);
            env.token0.transfer(&env.pair.address(), &input_amount);

            // Try with outputAmount + 1, should fail
            let result = env.pair.try_swap(
                output_amount + U256::one(),
                U256::zero(),
                env.owner,
                None,
            );
            assert!(result.is_err(), "Test case {} should fail with K error", i);

            // Should succeed with exact outputAmount
            env.pair
                .swap(output_amount, U256::zero(), env.owner, None);
        }
    }
}
