pub mod errors;

use odra::{casper_types::U256, prelude::*, ContractRef, uints::{ToU256, ToU512}};

use odra_modules::cep18_token::Cep18ContractRef;
use odra_modules::wrapped_native::WrappedNativeTokenContractRef;

use crate::{casperswap_v2_pair::CasperswapV2PairContractRef, factory::FactoryContractRef, router::errors::{CasperswapV2RouterError, CasperswapV2LibraryError}};

/// CasperswapV2Router - Router contract for CasperSwap V2
/// Based on UniswapV2Router02
#[odra::module]
pub struct CasperswapV2Router {
    factory: Var<Address>,
    wcspr: Var<Address>,
}

#[odra::module]
impl CasperswapV2Router {
    /// Initializes the router with the factory address
    pub fn init(&mut self, factory: Address, wcspr: Address) {
        self.factory.set(factory);
        self.wcspr.set(wcspr);
    }

    /// Returns the factory address
    pub fn factory(&self) -> Address {
        self.factory.get_or_revert_with(CasperswapV2RouterError::Misconfigured)
    }

    /// Returns the WCSPR address
    pub fn wcspr(&self) -> Address {
        self.wcspr.get_or_revert_with(CasperswapV2RouterError::Misconfigured)
    }

    /// Accepts CSPR deposits from WCSPR contract (equivalent to Solidity's receive() function)
    /// Only accepts CSPR from WCSPR contract
    #[odra(payable)]
    pub fn deposit(&self) {
        let wcspr = self.wcspr();
        if self.env().caller() != wcspr {
            self.env().revert(CasperswapV2RouterError::Misconfigured);
        }
    }

    // **** ADD LIQUIDITY ****

    /// Internal function to add liquidity
    fn _add_liquidity(
        &mut self,
        token_a: Address,
        token_b: Address,
        amount_a_desired: U256,
        amount_b_desired: U256,
        amount_a_min: U256,
        amount_b_min: U256,
    ) -> (U256, U256, CasperswapV2PairContractRef) {
        let pair = self.factory_instance().get_pair(token_a, token_b);
        let pair = pair.unwrap_or_else(|| self.factory_instance().create_pair(token_a, token_b));
        let pair_instance = CasperswapV2PairContractRef::new(self.env(), pair);
        let (reserve_a, reserve_b, _) = pair_instance.get_reserves();
        if reserve_a.is_zero() && reserve_b.is_zero() {
            (amount_a_desired, amount_b_desired, pair_instance)
        } else {
            let amount_b_optimal = self.quote(amount_a_desired, reserve_a, reserve_b);
            if amount_b_optimal <= amount_b_desired {
                if amount_b_optimal < amount_b_min {
                    self.env().revert(CasperswapV2RouterError::InsufficientBAmount);
                }
                (amount_a_desired, amount_b_optimal, pair_instance)
            } else {
                let amount_a_optimal = self.quote(amount_b_desired, reserve_b, reserve_a);
                if amount_a_optimal > amount_a_min {
                    self.env().revert(CasperswapV2RouterError::InsufficientAAmount);
                }
                (amount_a_optimal, amount_b_desired, pair_instance)
            }
        }
    }

    /// Add liquidity to a token pair
    pub fn add_liquidity(
        &mut self,
        token_a: Address,
        token_b: Address,
        amount_a_desired: U256,
        amount_b_desired: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        to: Address,
        deadline: u64,
    ) -> (U256, U256, U256) {
        if self.env().get_block_time() > deadline {
            self.env().revert(CasperswapV2RouterError::Expired);
        }

        let (amount_a, amount_b, mut pair_instance) = self._add_liquidity(token_a, token_b, amount_a_desired, amount_b_desired, amount_a_min, amount_b_min);
        
        let mut token_a_instance = Cep18ContractRef::new(self.env(), token_a);
        let mut token_b_instance = Cep18ContractRef::new(self.env(), token_b);

        token_a_instance.transfer_from(&self.env().caller(), &pair_instance.address(), &amount_a);
        token_b_instance.transfer_from(&self.env().caller(), &pair_instance.address(), &amount_b);
        let liquidity = pair_instance.mint(to);

        (amount_a, amount_b, liquidity)
    }

    /// Add liquidity to a token-CSPR pair
    #[odra(payable)]
    pub fn add_liquidity_cspr(
        &mut self,
        token: Address,
        amount_token_desired: U256,
        amount_token_min: U256,
        amount_cspr_min: U256,
        to: Address,
        deadline: u64,
    ) -> (U256, U256, U256) {
        if self.env().get_block_time() > deadline {
            self.env().revert(CasperswapV2RouterError::Expired);
        }

        let wcspr = self.wcspr();
        let cspr_amount = self.env().attached_value().to_u256().unwrap_or_revert(self);
        
        let (amount_token, amount_cspr, mut pair_instance) = self._add_liquidity(
            token,
            wcspr,
            amount_token_desired,
            cspr_amount,
            amount_token_min,
            amount_cspr_min,
        );
        
        // Transfer token from caller to pair
        let mut token_instance = Cep18ContractRef::new(self.env(), token);
        token_instance.transfer_from(&self.env().caller(), &pair_instance.address(), &amount_token);
        
        // Wrap CSPR and transfer to pair
        let mut wcspr_instance = self.wcspr_instance();
        // Pass CSPR tokens to the deposit call (like WETH.deposit{value: amountETH} in Solidity)
        wcspr_instance.with_tokens(amount_cspr.to_u512()).deposit();
        wcspr_instance.transfer(&pair_instance.address(), &amount_cspr);
        
        // Mint liquidity tokens
        let liquidity = pair_instance.mint(to);
        
        // Refund excess CSPR if any
        let excess_cspr = cspr_amount - amount_cspr;
        if excess_cspr > U256::from(0) {
            self.env().transfer_tokens(&self.env().caller(), &odra::uints::ToU512::to_u512(excess_cspr));
        }

        (amount_token, amount_cspr, liquidity)
    }

    // **** REMOVE LIQUIDITY ****

    /// Remove liquidity from a token pair
    pub fn remove_liquidity(
        &mut self,
        token_a: Address,
        token_b: Address,
        liquidity: U256,
        amount_a_min: U256,
        amount_b_min: U256,
        to: Address,
        deadline: u64,
    ) -> (U256, U256) {
        if self.env().get_block_time() > deadline {
            self.env().revert(CasperswapV2RouterError::Expired);
        }

        let pair_address = self.pair_for(token_a, token_b);
        let mut pair = CasperswapV2PairContractRef::new(self.env(), pair_address);
        
        // Transfer liquidity tokens to pair
        pair.transfer_from(&self.env().caller(), &pair_address, &liquidity);
        
        // Burn liquidity tokens and get back token amounts
        let (amount0, amount1) = pair.burn(to);
        
        // Sort amounts based on token order
        let (token0, _) = self.sort_tokens(token_a, token_b);
        let (amount_a, amount_b) = if token_a == token0 {
            (amount0, amount1)
        } else {
            (amount1, amount0)
        };
        
        // Verify minimum amounts
        if amount_a < amount_a_min {
            self.env().revert(CasperswapV2RouterError::InsufficientAAmount);
        }
        if amount_b < amount_b_min {
            self.env().revert(CasperswapV2RouterError::InsufficientBAmount);
        }
        
        (amount_a, amount_b)
    }


    /// Remove liquidity from a token-CSPR pair
    pub fn remove_liquidity_cspr(
        &mut self,
        token: Address,
        liquidity: U256,
        amount_token_min: U256,
        amount_cspr_min: U256,
        to: Address,
        deadline: u64,
    ) -> (U256, U256) {
        let wcspr = self.wcspr();
        let router_address = self.env().self_address();
        
        let (amount_token, amount_cspr) = self.remove_liquidity(
            token,
            wcspr,
            liquidity,
            amount_token_min,
            amount_cspr_min,
            router_address,
            deadline,
        );
        
        // Transfer token to recipient
        let mut token_instance = Cep18ContractRef::new(self.env(), token);
        token_instance.transfer(&to, &amount_token);
        
        // Withdraw CSPR from WCSPR and transfer to recipient
        let mut wcspr_instance = self.wcspr_instance();
        wcspr_instance.withdraw(&amount_cspr);
        self.env().transfer_tokens(&to, &odra::uints::ToU512::to_u512(amount_cspr));
        
        (amount_token, amount_cspr)
    }


    // **** SWAP ****

    /// Internal swap function - requires the initial amount to have already been sent to the first pair
    fn _swap(&mut self, amounts: Vec<U256>, path: Vec<Address>, to: Address) {
        for i in 0..path.len() - 1 {
            let input = path[i];
            let output = path[i + 1];
            let (token0, _) = self.sort_tokens(input, output);
            let amount_out = amounts[i + 1];
            let (amount0_out, amount1_out) = if input == token0 {
                (U256::zero(), amount_out)
            } else {
                (amount_out, U256::zero())
            };
            let recipient = if i < path.len() - 2 {
                self.pair_for(output, path[i + 2])
            } else {
                to
            };
            let pair_address = self.pair_for(input, output);
            let mut pair = CasperswapV2PairContractRef::new(self.env(), pair_address);
            pair.swap(amount0_out, amount1_out, recipient, None);
        }
    }

    /// Swap exact tokens for tokens
    pub fn swap_exact_tokens_for_tokens(
        &mut self,
        amount_in: U256,
        amount_out_min: U256,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Vec<U256> {
        // Check deadline
        if self.env().get_block_time() > deadline {
            self.env().revert(CasperswapV2RouterError::Expired);
        }

        // Calculate amounts
        let amounts = self.get_amounts_out(amount_in, path.clone());
        if amounts[amounts.len() - 1] < amount_out_min {
            self.env().revert(CasperswapV2RouterError::InsufficientOutputAmount);
        }

        // Transfer input tokens to first pair
        let pair_address = self.pair_for(path[0], path[1]);
        let mut token_instance = Cep18ContractRef::new(self.env(), path[0]);
        token_instance.transfer_from(&self.env().caller(), &pair_address, &amounts[0]);

        // Perform swap
        self._swap(amounts.clone(), path, to);

        amounts
    }

    /// Swap tokens for exact tokens
    pub fn swap_tokens_for_exact_tokens(
        &mut self,
        amount_out: U256,
        amount_in_max: U256,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Vec<U256> {
        // Check deadline
        if self.env().get_block_time() > deadline {
            self.env().revert(CasperswapV2RouterError::Expired);
        }

        // Calculate amounts
        let amounts = self.get_amounts_in(amount_out, path.clone());
        if amounts[0] > amount_in_max {
            self.env().revert(CasperswapV2RouterError::ExcessiveInputAmount);
        }

        // Transfer input tokens to first pair
        let pair_address = self.pair_for(path[0], path[1]);
        let mut token_instance = Cep18ContractRef::new(self.env(), path[0]);
        token_instance.transfer_from(&self.env().caller(), &pair_address, &amounts[0]);

        // Perform swap
        self._swap(amounts.clone(), path, to);

        amounts
    }


    /// Given some amount of an asset and pair reserves, returns an equivalent amount of the other asset
    pub fn quote(&self, amount_a: U256, reserve_a: U256, reserve_b: U256) -> U256 {
        if amount_a.is_zero() {
            self.env().revert(CasperswapV2LibraryError::InsufficientAmount);
        }
        if reserve_a.is_zero() || reserve_b.is_zero() {
            self.env().revert(CasperswapV2LibraryError::InsufficientLiquidity);
        }
        amount_a * reserve_b / reserve_a
    }

    /// Given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset
    pub fn get_amount_out(&self, amount_in: U256, reserve_in: U256, reserve_out: U256) -> U256 {
        if amount_in.is_zero() {
            self.env().revert(CasperswapV2LibraryError::InsufficientInputAmount);
        }
        if reserve_in.is_zero() || reserve_out.is_zero() {
            self.env().revert(CasperswapV2LibraryError::InsufficientLiquidity);
        }
        let amount_in_with_fee = amount_in * U256::from(997);
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in * U256::from(1000) + amount_in_with_fee;
        numerator / denominator
    }

    /// Given an output amount of an asset and pair reserves, returns a required input amount of the other asset
    pub fn get_amount_in(&self, amount_out: U256, reserve_in: U256, reserve_out: U256) -> U256 {
        if amount_out.is_zero() {
            self.env().revert(CasperswapV2LibraryError::InsufficientOutputAmount);
        }
        if reserve_in.is_zero() || reserve_out.is_zero() {
            self.env().revert(CasperswapV2LibraryError::InsufficientLiquidity);
        }
        let numerator = reserve_in * amount_out * U256::from(1000);
        let denominator = (reserve_out - amount_out) * U256::from(997);
        (numerator / denominator) + U256::from(1)
    }

    /// Performs chained getAmountOut calculations on any number of pairs
    pub fn get_amounts_out(&self, amount_in: U256, path: Vec<Address>) -> Vec<U256> {
        if path.len() < 2 {
            self.env().revert(CasperswapV2LibraryError::InvalidPath);
        }
        let mut amounts = vec![U256::zero(); path.len()];
        amounts[0] = amount_in;
        for i in 0..path.len() - 1 {
            let (reserve_in, reserve_out, _) = self.get_reserves(path[i], path[i + 1]);
            amounts[i + 1] = self.get_amount_out(amounts[i], reserve_in, reserve_out);
        }
        amounts
    }

    /// Performs chained getAmountIn calculations on any number of pairs
    pub fn get_amounts_in(&self, amount_out: U256, path: Vec<Address>) -> Vec<U256> {
        if path.len() < 2 {
            self.env().revert(CasperswapV2LibraryError::InvalidPath);
        }
        let mut amounts = vec![U256::zero(); path.len()];
        let len = amounts.len();
        amounts[len - 1] = amount_out;
        for i in (1..path.len()).rev() {
            let (reserve_in, reserve_out, _) = self.get_reserves(path[i - 1], path[i]);
            let current_amount = amounts[i];
            amounts[i - 1] = self.get_amount_in(current_amount, reserve_in, reserve_out);
        }
        amounts
    }

}

impl CasperswapV2Router {
    fn factory_instance(&self) -> FactoryContractRef {
        FactoryContractRef::new(self.env(), self.factory())
    }

    fn wcspr_instance(&self) -> WrappedNativeTokenContractRef {
        WrappedNativeTokenContractRef::new(self.env(), self.wcspr())
    }

    /// Fetches and sorts the reserves for a pair
    fn get_reserves(&self, token_a: Address, token_b: Address) -> (U256, U256, Address) {
        let (token0, _) = self.sort_tokens(token_a, token_b);
        let pair_address = self.pair_for(token_a, token_b);
        let pair = CasperswapV2PairContractRef::new(self.env(), pair_address);
        let (reserve0, reserve1, _) = pair.get_reserves();
        let (reserve_a, reserve_b) = if token_a == token0 {
            (reserve0, reserve1)
        } else {
            (reserve1, reserve0)
        };
        (reserve_a, reserve_b, pair_address)
    }

    /// Returns sorted token addresses, used to handle return values from pairs sorted in this order
    fn sort_tokens(&self, token_a: Address, token_b: Address) -> (Address, Address) {
        if token_a == token_b {
            self.env().revert(CasperswapV2LibraryError::IdenticalAddresses);
        }
        let (token0, token1) = if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        };
        // Check if token0 is zero address
        if token0 == crate::utils::zero_address() {
            self.env().revert(CasperswapV2LibraryError::ZeroAddress);
        }
        (token0, token1)
    }

    /// Calculates the pair address for a pair
    fn pair_for(&self, token_a: Address, token_b: Address) -> Address {
        // In Uniswap V2, the pair address is calculated, but in Casper we get the address during the deployment
        // So we get the pair address from the factory
        self.factory_instance().get_pair(token_a, token_b).unwrap_or_revert_with(&self.env(), errors::CasperswapV2RouterError::PairNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        casperswap_v2_pair::{CasperswapV2Pair, CasperswapV2PairHostRef, CasperswapV2PairInitArgs, MINIMUM_LIQUIDITY}, factory::{Factory, FactoryHostRef, FactoryInitArgs}, sample_tokens::{SampleToken, SampleTokenHostRef, SampleTokenInitArgs}, utils::{expand_to_18_decimals, expand_to_9_decimals}
    };
    use odra::{
        host::{Deployer, HostEnv, HostRef, NoArgs},
        prelude::Address,
    };
    use odra_modules::{cep18_token::Cep18HostRef, wrapped_native::{WrappedNativeToken, WrappedNativeTokenHostRef}};

    struct RouterEnv {
        pub odra_env: HostEnv,
        pub router: CasperswapV2RouterHostRef,
        pub factory: FactoryHostRef,
        pub token0: SampleTokenHostRef,
        pub token1: SampleTokenHostRef,
        pub wcspr: WrappedNativeTokenHostRef,
        pub wcspr_partner: SampleTokenHostRef,
        pub pair: CasperswapV2PairHostRef,
        pub wcspr_pair: CasperswapV2PairHostRef,
        pub owner: Address,
        pub alice: Address,
        pub bob: Address,
    }

    fn setup_router() -> RouterEnv {
        let env = odra_test::env();
        let owner = env.get_account(0);
        let alice = env.get_account(1);
        let bob = env.get_account(2);
        
        // Deploy the actual Factory contract
        let mut factory = Factory::deploy(&env, FactoryInitArgs {
            fee_to: None,
        });
        
        // Deploy WCSPR contract
        let wcspr = WrappedNativeToken::deploy(&env, NoArgs);
        
        // Deploy Router with the factory and wcspr address
        let router = CasperswapV2Router::deploy(&env, CasperswapV2RouterInitArgs {
            factory: factory.address(),
            wcspr: wcspr.address(),
        });

        
        // Deploy tokens
        let token0 = SampleToken::deploy(
            &env,
            SampleTokenInitArgs {
                name: "Sample Token A".to_string(),
                symbol: "STA".to_string(),
                decimals: 18,
                initial_supply: expand_to_18_decimals(10000),
            },
        );

        let token1 = SampleToken::deploy(
            &env,
            SampleTokenInitArgs {
                name: "Sample Token B".to_string(),
                symbol: "STB".to_string(),
                decimals: 18,
                initial_supply: expand_to_18_decimals(10000),
            },
        );

        // Deploy WCSPR partner token (equivalent to WETHPartner in fixtures.ts)
        let wcspr_partner = SampleToken::deploy(
            &env,
            SampleTokenInitArgs {
                name: "WETH Partner".to_string(),
                symbol: "WETHP".to_string(),
                decimals: 18,
                initial_supply: expand_to_18_decimals(10000),
            },
        );

        // Deploy pair for token0-token1
        let mut pair = CasperswapV2Pair::deploy(&env, CasperswapV2PairInitArgs {
            factory: factory.address(),
        });
        pair.initialize(token0.address(), token1.address());

        // Deploy pair for WCSPR-WCSPRPartner
        let mut wcspr_pair = CasperswapV2Pair::deploy(&env, CasperswapV2PairInitArgs {
            factory: factory.address(),
        });
        wcspr_pair.initialize(wcspr.address(), wcspr_partner.address());

        // Make factory return the pairs
        factory.will_return_pair(Some(pair.address()));
        
        RouterEnv {
            odra_env: env,
            router,
            factory,
            token0,
            token1,
            wcspr,
            wcspr_partner,
            pair,
            wcspr_pair,
            owner,
            alice,
            bob,
        }
    }

    #[test]
    fn test_quote() {

        let env = setup_router();
        
        // Test basic quote functionality
        assert_eq!(
            env.router.quote(U256::from(1), U256::from(100), U256::from(200)),
            U256::from(2)
        );
        assert_eq!(
            env.router.quote(U256::from(2), U256::from(200), U256::from(100)),
            U256::from(1)
        );
        
        // Test error cases
        assert_eq!(
            env.router
                .try_quote(U256::from(0), U256::from(100), U256::from(200))
                .unwrap_err(),
            CasperswapV2LibraryError::InsufficientAmount.into()
        );
        assert_eq!(
            env.router
                .try_quote(U256::from(1), U256::from(0), U256::from(200))
                .unwrap_err(),
            CasperswapV2LibraryError::InsufficientLiquidity.into()
        );
        assert_eq!(
            env.router
                .try_quote(U256::from(1), U256::from(100), U256::from(0))
                .unwrap_err(),
            CasperswapV2LibraryError::InsufficientLiquidity.into()
        );
    }

    #[test]
    fn test_get_amount_out() {
        let env = setup_router();
        
        // Test basic getAmountOut functionality
        // With 0.3% fee: input 2, reserves 100/100, expect output ~1
        assert_eq!(
            env.router.get_amount_out(U256::from(2), U256::from(100), U256::from(100)),
            U256::from(1)
        );
        
        // Test error cases
        assert_eq!(
            env.router
                .try_get_amount_out(U256::from(0), U256::from(100), U256::from(100))
                .unwrap_err(),
            CasperswapV2LibraryError::InsufficientInputAmount.into()
        );
        assert_eq!(
            env.router
                .try_get_amount_out(U256::from(2), U256::from(0), U256::from(100))
                .unwrap_err(),
            CasperswapV2LibraryError::InsufficientLiquidity.into()
        );
        assert_eq!(
            env.router
                .try_get_amount_out(U256::from(2), U256::from(100), U256::from(0))
                .unwrap_err(),
            CasperswapV2LibraryError::InsufficientLiquidity.into()
        );
    }

    #[test]
    fn test_get_amount_in() {
        let env = setup_router();
        
        // Test basic getAmountIn functionality
        // With 0.3% fee: output 1, reserves 100/100, expect input ~2
        assert_eq!(
            env.router.get_amount_in(U256::from(1), U256::from(100), U256::from(100)),
            U256::from(2)
        );
        
        // Test error cases
        assert_eq!(
            env.router
                .try_get_amount_in(U256::from(0), U256::from(100), U256::from(100))
                .unwrap_err(),
            CasperswapV2LibraryError::InsufficientOutputAmount.into()
        );
        assert_eq!(
            env.router
                .try_get_amount_in(U256::from(1), U256::from(0), U256::from(100))
                .unwrap_err(),
            CasperswapV2LibraryError::InsufficientLiquidity.into()
        );
        assert_eq!(
            env.router
                .try_get_amount_in(U256::from(1), U256::from(100), U256::from(0))
                .unwrap_err(),
            CasperswapV2LibraryError::InsufficientLiquidity.into()
        );
    }

    #[test]
    fn test_get_amounts_out() {
        let mut env = setup_router();
        env.token0.approve(&env.router.address(), &U256::from(10000));
        env.token1.approve(&env.router.address(), &U256::from(10000));
        env.router.add_liquidity(env.token0.address(), env.token1.address(), U256::from(10000), U256::from(10000), U256::from(0), U256::from(0), env.alice, 0);
        
        // Test invalid path (single token)
        let invalid_path = vec![env.token0.address()];
        assert_eq!(
            env.router
                .try_get_amounts_out(U256::from(2), invalid_path)
                .unwrap_err(),
            CasperswapV2LibraryError::InvalidPath.into()
        );

        let path = vec![env.token0.address(), env.token1.address()];
        assert_eq!(
            env.router
                .try_get_amounts_out(U256::from(2), path)
                .unwrap(),
            vec![U256::from(2), U256::from(1)]
        );
    }

    #[test]
    fn test_get_amounts_in() {
        let mut env = setup_router();
        env.token0.approve(&env.router.address(), &U256::from(10000));
        env.token1.approve(&env.router.address(), &U256::from(10000));
        env.router.add_liquidity(env.token0.address(), env.token1.address(), U256::from(10000), U256::from(10000), U256::from(0), U256::from(0), env.alice, 0);
        
        // Test invalid path (single token)
        let invalid_path = vec![env.token0.address()];
        assert_eq!(
            env.router
                .try_get_amounts_in(U256::from(1), invalid_path)
                .unwrap_err(),
            CasperswapV2LibraryError::InvalidPath.into()
        );

        let path = vec![env.token0.address(), env.token1.address()];
        assert_eq!(
            env.router
                .try_get_amounts_in(U256::from(1), path)
                .unwrap(),
            vec![U256::from(2), U256::from(1)]
        );
    }

    #[test]
    fn test_add_liquidity() {
        let mut env = setup_router();

        let token0_amount = expand_to_18_decimals(1);
        let token1_amount = expand_to_18_decimals(4);

        let expected_liquidity = expand_to_18_decimals(2);
        env.token0.approve(&env.router.address(), &U256::MAX);
        env.token1.approve(&env.router.address(), &U256::MAX);

        let (amount0, amount1, liquidity) = env.router.add_liquidity(
            env.token0.address(),
            env.token1.address(),
            token0_amount,
            token1_amount,
            U256::from(0), // amountAMin
            U256::from(0), // amountBMin
            env.owner,
            u64::MAX, // deadline
        );

        assert_eq!(liquidity, expected_liquidity - U256::from(MINIMUM_LIQUIDITY));
    }

    // great name from uniswap tests
    fn add_liquidity(env: &mut RouterEnv, token0_amount: U256, token1_amount: U256) {
        let pair_address = env.pair.address();
        let mut pair_instance = CasperswapV2PairHostRef::new(pair_address, env.odra_env.clone());
        let mut token0_instance = Cep18HostRef::new(pair_instance.token0(), env.odra_env.clone());
        let mut token1_instance = Cep18HostRef::new(pair_instance.token1(), env.odra_env.clone());
        token0_instance.transfer(&pair_instance.address(), &token0_amount);
        token1_instance.transfer(&pair_instance.address(), &token1_amount);

        pair_instance.mint(env.owner);
    }

    #[test]
    fn test_add_liquidity_cspr() {
        let mut env = setup_router();

        let mut cspr_pair = CasperswapV2Pair::deploy(&env.odra_env, CasperswapV2PairInitArgs {
            factory: env.factory.address(),
        });
        cspr_pair.initialize(env.token0.address(), env.wcspr.address());

        env.factory.will_return_pair(Some(cspr_pair.address()));

        let token_amount = expand_to_18_decimals(1);
        let cspr_amount = expand_to_9_decimals(4);

        // DIFFERENCE FROM UNISWAP: Liquidity calculation differs due to different decimals
        // Uniswap: expectedLiquidity = expandTo18Decimals(2) (simple 2 * 10^18)
        // Our case: sqrt(1 * 10^18 * 4 * 10^9) = sqrt(4 * 10^27) = 2 * 10^13.5 ≈ 63,245,553,202,367
        let expected_liquidity = (token_amount * cspr_amount).integer_sqrt() - U256::from(MINIMUM_LIQUIDITY);

        env.token0.approve(&env.router.address(), &U256::MAX);

        let (amount_token, amount_cspr, liquidity) = env.router
            .with_tokens(cspr_amount.to_u512())
            .add_liquidity_cspr(
                env.token0.address(),
                token_amount,
                U256::from(0),
                U256::from(0),
                env.owner,
                u64::MAX,
            );

        assert_eq!(amount_token, token_amount);
        assert_eq!(amount_cspr, cspr_amount);
        assert_eq!(liquidity, expected_liquidity);
    }

    #[test]
    fn test_remove_liquidity() {
        let mut env = setup_router();

        let token0_amount = expand_to_18_decimals(1);
        let token1_amount = expand_to_18_decimals(4);
        add_liquidity(&mut env, token0_amount, token1_amount);

        let expected_liquidity = expand_to_18_decimals(2);

        // Approve router to spend pair tokens
        env.pair.approve(&env.router.address(), &U256::MAX);

        // Remove liquidity - should get back less than original amounts due to 0.3% fee
        env.router.remove_liquidity(
            env.token0.address(),
            env.token1.address(),
            expected_liquidity - U256::from(MINIMUM_LIQUIDITY),
            U256::from(0),
            U256::from(0),
            env.owner,
            u64::MAX,
        );

        assert_eq!(env.pair.balance_of(&env.owner), U256::from(0));
        let total_supply_token0 = env.token0.total_supply();
        let total_supply_token1 = env.token1.total_supply();
        assert_eq!(env.token0.balance_of(&env.owner), total_supply_token0 - U256::from(500));
        assert_eq!(env.token1.balance_of(&env.owner), total_supply_token1 - U256::from(2000));

    }

    #[test]
    fn test_remove_liquidity_cspr() {
        let mut env = setup_router();

        let wcspr_partner_amount = expand_to_18_decimals(1);
        let cspr_amount = expand_to_18_decimals(4);

        env.factory.will_return_pair(Some(env.wcspr_pair.address()));

        env.wcspr_partner.transfer(&env.wcspr_pair.address(), &wcspr_partner_amount);
        env.wcspr.with_tokens(cspr_amount.to_u512()).deposit();
        env.wcspr.transfer(&env.wcspr_pair.address(), &cspr_amount);
        env.wcspr_pair.mint(env.owner);

        let expected_liquidity = expand_to_18_decimals(2);
        env.wcspr_pair.approve(&env.router.address(), &U256::MAX);

        let (amount_token, amount_cspr) = env.router.remove_liquidity_cspr(
            env.wcspr_partner.address(),
            expected_liquidity - U256::from(MINIMUM_LIQUIDITY),
            U256::from(0),
            U256::from(0),
            env.owner,
            u64::MAX,
        );

        assert_eq!(env.wcspr_pair.balance_of(&env.owner), U256::from(0));
        let total_supply_wcspr_partner = env.wcspr_partner.total_supply();
        let total_supply_wcspr = env.wcspr.total_supply();
        assert_eq!(env.wcspr_partner.balance_of(&env.owner), total_supply_wcspr_partner - U256::from(500));
        assert_eq!(env.wcspr.balance_of(&env.owner), total_supply_wcspr - U256::from(2000));
    }

    #[test]
    fn test_swap_exact_tokens_for_tokens_happy_path() {
        let mut env = setup_router();

        let token0_amount = expand_to_18_decimals(5);
        let token1_amount = expand_to_18_decimals(10);
        let swap_amount = expand_to_18_decimals(1);
        let expected_output_amount = U256::from_dec_str("1662497915624478906").unwrap();

        // Add liquidity first
        add_liquidity(&mut env, token0_amount, token1_amount);

        // Approve router to spend tokens
        env.token0.approve(&env.router.address(), &U256::MAX);

        // Perform swap
        env.router.swap_exact_tokens_for_tokens(
            swap_amount,
            U256::from(0),
            vec![env.token0.address(), env.token1.address()],
            env.owner,
            u64::MAX,
        );

        // Verify events were emitted
        // Check token0 TransferFrom event (router calls transfer_from from owner to pair)
        use odra_modules::cep18::events::{Transfer, TransferFrom};
        assert!(env.odra_env.emitted_event(
            &env.token0,
            TransferFrom {
                spender: env.router.address(),
                owner: env.owner,
                recipient: env.pair.address(),
                amount: swap_amount,
            }
        ));
        
        // Check token1 Transfer event (pair sends output to owner)
        assert!(env.odra_env.emitted_event(
            &env.token1,
            Transfer {
                sender: env.pair.address(),
                recipient: env.owner,
                amount: expected_output_amount,
            }
        ));
        
        // Check Sync event
        assert!(env.odra_env.emitted_event(
            &env.pair,
            crate::casperswap_v2_pair::events::Sync {
                reserve0: token0_amount + swap_amount,
                reserve1: token1_amount - expected_output_amount,
            }
        ));
        
        // Check Swap event
        assert!(env.odra_env.emitted_event(
            &env.pair,
            crate::casperswap_v2_pair::events::Swap {
                sender: env.router.address(),
                amount0_in: swap_amount,
                amount1_in: U256::zero(),
                amount0_out: U256::zero(),
                amount1_out: expected_output_amount,
                to: env.owner,
            }
        ));
    }

    #[test]
    fn test_swap_exact_tokens_for_tokens_amounts() {
        let mut env = setup_router();

        let token0_amount = expand_to_18_decimals(5);
        let token1_amount = expand_to_18_decimals(10);
        let swap_amount = expand_to_18_decimals(1);
        let expected_output_amount = U256::from_dec_str("1662497915624478906").unwrap();

        // Add liquidity first
        add_liquidity(&mut env, token0_amount, token1_amount);

        // Approve router to spend tokens
        env.token0.approve(&env.router.address(), &U256::MAX);

        // Perform swap and verify returned amounts
        let amounts = env.router.swap_exact_tokens_for_tokens(
            swap_amount,
            U256::from(0),
            vec![env.token0.address(), env.token1.address()],
            env.owner,
            u64::MAX,
        );

        // Verify returned amounts match expected values
        assert_eq!(amounts, vec![swap_amount, expected_output_amount]);
    }

}

