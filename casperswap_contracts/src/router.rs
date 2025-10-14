pub mod errors;

use odra::{casper_types::U256, prelude::*, ContractRef};

// Library error types moved from casperswap_v2_library
pub mod library_errors {
    use odra::prelude::*;

    #[odra::odra_error]
    pub enum CasperswapV2LibraryError {
        IdenticalAddresses = 1,
        ZeroAddress = 2,
        InsufficientAmount = 3,
        InsufficientLiquidity = 4,
        InsufficientInputAmount = 5,
        InsufficientOutputAmount = 6,
        InvalidPath = 7,
    }
}
use odra_modules::cep18_token::Cep18ContractRef;

use crate::{casperswap_v2_pair::CasperswapV2PairContractRef, factory::FactoryContractRef, router::errors::CasperswapV2RouterError};

/// CasperswapV2Router - Router contract for CasperSwap V2
/// Based on UniswapV2Router02
#[odra::module]
pub struct CasperswapV2Router {
    factory: Var<Address>,
}

#[odra::module]
impl CasperswapV2Router {
    /// Initializes the router with the factory address
    pub fn init(&mut self, factory: Address) {
        self.factory.set(factory);
    }

    /// Returns the factory address
    pub fn factory(&self) -> Address {
        self.factory.get_or_revert_with(CasperswapV2RouterError::Misconfigured)
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

    // **** REMOVE LIQUIDITY ****

    /// Remove liquidity from a token pair
    pub fn remove_liquidity(
        &mut self,
        _token_a: Address,
        _token_b: Address,
        _liquidity: U256,
        _amount_a_min: U256,
        _amount_b_min: U256,
        _to: Address,
        _deadline: u64,
    ) -> (U256, U256) {
        // TODO: Implement remove_liquidity
        // 1. Check deadline
        // 2. Transfer liquidity tokens to pair
        // 3. Call pair.burn()
        // 4. Verify minimum amounts
        unimplemented!("remove_liquidity")
    }

    /// Remove liquidity with permit (gasless approval)
    pub fn remove_liquidity_with_permit(
        &mut self,
        _token_a: Address,
        _token_b: Address,
        _liquidity: U256,
        _amount_a_min: U256,
        _amount_b_min: U256,
        _to: Address,
        _deadline: u64,
        _approve_max: bool,
        _v: u8,
        _r: [u8; 32],
        _s: [u8; 32],
    ) -> (U256, U256) {
        // TODO: Implement remove_liquidity_with_permit
        // 1. Get pair address
        // 2. Call permit on pair
        // 3. Call remove_liquidity
        unimplemented!("remove_liquidity_with_permit")
    }

    // **** SWAP ****

    /// Internal swap function - requires the initial amount to have already been sent to the first pair
    fn _swap(&mut self, _amounts: Vec<U256>, _path: Vec<Address>, _to: Address) {
        // TODO: Implement _swap
        // Loop through path and perform swaps on each pair
        unimplemented!("_swap")
    }

    /// Swap exact tokens for tokens
    pub fn swap_exact_tokens_for_tokens(
        &mut self,
        _amount_in: U256,
        _amount_out_min: U256,
        _path: Vec<Address>,
        _to: Address,
        _deadline: u64,
    ) -> Vec<U256> {
        // TODO: Implement swap_exact_tokens_for_tokens
        // 1. Check deadline
        // 2. Calculate amounts using library
        // 3. Verify output amount
        // 4. Transfer input tokens to first pair
        // 5. Perform swap
        unimplemented!("swap_exact_tokens_for_tokens")
    }

    /// Swap tokens for exact tokens
    pub fn swap_tokens_for_exact_tokens(
        &mut self,
        _amount_out: U256,
        _amount_in_max: U256,
        _path: Vec<Address>,
        _to: Address,
        _deadline: u64,
    ) -> Vec<U256> {
        // TODO: Implement swap_tokens_for_exact_tokens
        // 1. Check deadline
        // 2. Calculate amounts using library
        // 3. Verify input amount
        // 4. Transfer input tokens to first pair
        // 5. Perform swap
        unimplemented!("swap_tokens_for_exact_tokens")
    }

    // **** SWAP (supporting fee-on-transfer tokens) ****

    /// Internal swap function supporting fee-on-transfer tokens
    fn _swap_supporting_fee_on_transfer_tokens(&mut self, _path: Vec<Address>, _to: Address) {
        // TODO: Implement _swap_supporting_fee_on_transfer_tokens
        // Similar to _swap but handles fee-on-transfer tokens
        unimplemented!("_swap_supporting_fee_on_transfer_tokens")
    }

    /// Swap exact tokens for tokens supporting fee-on-transfer tokens
    pub fn swap_exact_tokens_for_tokens_supporting_fee_on_transfer_tokens(
        &mut self,
        _amount_in: U256,
        _amount_out_min: U256,
        _path: Vec<Address>,
        _to: Address,
        _deadline: u64,
    ) {
        // TODO: Implement swap_exact_tokens_for_tokens_supporting_fee_on_transfer_tokens
        unimplemented!("swap_exact_tokens_for_tokens_supporting_fee_on_transfer_tokens")
    }

    // **** LIBRARY FUNCTIONS ****
    // These are moved from casperswap_v2_library to avoid passing env around

    /// Given some amount of an asset and pair reserves, returns an equivalent amount of the other asset
    pub fn quote(&self, amount_a: U256, reserve_a: U256, reserve_b: U256) -> U256 {
        if amount_a.is_zero() {
            self.env().revert(library_errors::CasperswapV2LibraryError::InsufficientAmount);
        }
        if reserve_a.is_zero() || reserve_b.is_zero() {
            self.env().revert(library_errors::CasperswapV2LibraryError::InsufficientLiquidity);
        }
        amount_a * reserve_b / reserve_a
    }

    /// Given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset
    pub fn get_amount_out(&self, amount_in: U256, reserve_in: U256, reserve_out: U256) -> U256 {
        if amount_in.is_zero() {
            self.env().revert(library_errors::CasperswapV2LibraryError::InsufficientInputAmount);
        }
        if reserve_in.is_zero() || reserve_out.is_zero() {
            self.env().revert(library_errors::CasperswapV2LibraryError::InsufficientLiquidity);
        }
        let amount_in_with_fee = amount_in * U256::from(997);
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in * U256::from(1000) + amount_in_with_fee;
        numerator / denominator
    }

    /// Given an output amount of an asset and pair reserves, returns a required input amount of the other asset
    pub fn get_amount_in(&self, amount_out: U256, reserve_in: U256, reserve_out: U256) -> U256 {
        if amount_out.is_zero() {
            self.env().revert(library_errors::CasperswapV2LibraryError::InsufficientOutputAmount);
        }
        if reserve_in.is_zero() || reserve_out.is_zero() {
            self.env().revert(library_errors::CasperswapV2LibraryError::InsufficientLiquidity);
        }
        let numerator = reserve_in * amount_out * U256::from(1000);
        let denominator = (reserve_out - amount_out) * U256::from(997);
        (numerator / denominator) + U256::from(1)
    }

    /// Performs chained getAmountOut calculations on any number of pairs
    pub fn get_amounts_out(&self, amount_in: U256, path: Vec<Address>) -> Vec<U256> {
        if path.len() < 2 {
            self.env().revert(library_errors::CasperswapV2LibraryError::InvalidPath);
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
            self.env().revert(library_errors::CasperswapV2LibraryError::InvalidPath);
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

    /// Fetches and sorts the reserves for a pair
    fn get_reserves(&self, token_a: Address, token_b: Address) -> (U256, U256, Address) {
        let (token0, _token1) = self.sort_tokens(token_a, token_b);
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
            self.env().revert(library_errors::CasperswapV2LibraryError::IdenticalAddresses);
        }
        let (token0, token1) = if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        };
        // Check if token0 is zero address
        if token0 == crate::utils::zero_address() {
            self.env().revert(library_errors::CasperswapV2LibraryError::ZeroAddress);
        }
        (token0, token1)
    }

    /// Calculates the pair address for a pair
    fn pair_for(&self, _token_a: Address, _token_b: Address) -> Address {
        // TODO: Implement pair address calculation
        // For now, we'll need to call the factory's get_pair method
        // This is a placeholder that will be updated when we implement the factory's get_pair
        unimplemented!("pair_for needs factory.get_pair() implementation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        casperswap_v2_pair::{CasperswapV2Pair, CasperswapV2PairInitArgs}, factory::{Factory, FactoryHostRef, FactoryInitArgs}, sample_tokens::{SampleToken, SampleTokenHostRef, SampleTokenInitArgs}, utils::expand_to_18_decimals
    };
    use odra::{
        host::{Deployer, HostEnv},
        prelude::Address,
    };

    struct RouterEnv {
        pub odra_env: HostEnv,
        pub router: CasperswapV2RouterHostRef,
        pub factory: FactoryHostRef,
        pub token0: SampleTokenHostRef,
        pub token1: SampleTokenHostRef,
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
        
        // Deploy Router with the factory address
        let router = CasperswapV2Router::deploy(&env, CasperswapV2RouterInitArgs {
            factory: factory.address(),
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

        // Deploy pair mock for our tests, until the factory is implemented
        let mut pair = CasperswapV2Pair::deploy(&env, CasperswapV2PairInitArgs {
            factory: factory.address(),
        });

        pair.initialize(token0.address(), token1.address());

        // Make factory return the pair
        factory.will_return_pair(Some(pair.address()));
        
        RouterEnv {
            odra_env: env,
            router,
            factory,
            token0,
            token1,
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
            library_errors::CasperswapV2LibraryError::InsufficientAmount.into()
        );
        assert_eq!(
            env.router
                .try_quote(U256::from(1), U256::from(0), U256::from(200))
                .unwrap_err(),
            library_errors::CasperswapV2LibraryError::InsufficientLiquidity.into()
        );
        assert_eq!(
            env.router
                .try_quote(U256::from(1), U256::from(100), U256::from(0))
                .unwrap_err(),
            library_errors::CasperswapV2LibraryError::InsufficientLiquidity.into()
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
            library_errors::CasperswapV2LibraryError::InsufficientInputAmount.into()
        );
        assert_eq!(
            env.router
                .try_get_amount_out(U256::from(2), U256::from(0), U256::from(100))
                .unwrap_err(),
            library_errors::CasperswapV2LibraryError::InsufficientLiquidity.into()
        );
        assert_eq!(
            env.router
                .try_get_amount_out(U256::from(2), U256::from(100), U256::from(0))
                .unwrap_err(),
            library_errors::CasperswapV2LibraryError::InsufficientLiquidity.into()
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
            library_errors::CasperswapV2LibraryError::InsufficientOutputAmount.into()
        );
        assert_eq!(
            env.router
                .try_get_amount_in(U256::from(1), U256::from(0), U256::from(100))
                .unwrap_err(),
            library_errors::CasperswapV2LibraryError::InsufficientLiquidity.into()
        );
        assert_eq!(
            env.router
                .try_get_amount_in(U256::from(1), U256::from(100), U256::from(0))
                .unwrap_err(),
            library_errors::CasperswapV2LibraryError::InsufficientLiquidity.into()
        );
    }

    #[test]
    fn test_get_amounts_out() {
        let env = setup_router();
        
        // Test invalid path (single token)
        let invalid_path = vec![env.token0.address()];
        assert_eq!(
            env.router
                .try_get_amounts_out(U256::from(2), invalid_path)
                .unwrap_err(),
            library_errors::CasperswapV2LibraryError::InvalidPath.into()
        );
        
        // Note: For now, we can't test the success case because get_amounts_out
        // requires factory.get_pair() and pair.get_reserves() which are not yet implemented.
        // The success case will be: 
        // let path = vec![env.token0.address(), env.token1.address()];
        // let amounts = env.router.get_amounts_out(U256::from(2), path);
        // assert_eq!(amounts, vec![U256::from(2), U256::from(1)]);
        // This will be implemented when we have full factory/pair support.
    }

    #[test]
    fn test_get_amounts_in() {
        let env = setup_router();
        
        // Test invalid path (single token)
        let invalid_path = vec![env.token0.address()];
        assert_eq!(
            env.router
                .try_get_amounts_in(U256::from(1), invalid_path)
                .unwrap_err(),
            library_errors::CasperswapV2LibraryError::InvalidPath.into()
        );
        
        // Note: For now, we can't test the success case because get_amounts_in
        // requires factory.get_pair() and pair.get_reserves() which are not yet implemented.
        // The success case will be: 
        // let path = vec![env.token0.address(), env.token1.address()];
        // let amounts = env.router.get_amounts_in(U256::from(1), path);
        // assert_eq!(amounts, vec![U256::from(2), U256::from(1)]);
        // This will be implemented when we have full factory/pair support.
    }
}

