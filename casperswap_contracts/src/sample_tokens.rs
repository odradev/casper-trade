//! Sample CEP-18 token for testing CasperswapV2Pair
use odra::casper_types::U256;
use odra::prelude::*;
use odra_modules::access::Ownable;
use odra_modules::cep18_token::Cep18;

/// Sample Token - A simple CEP-18 token with minting capability
/// Can be deployed multiple times with different names and symbols
#[odra::module]
pub struct SampleToken {
    token: SubModule<Cep18>,
    ownable: SubModule<Ownable>,
}

#[odra::module]
impl SampleToken {
    /// Initialize the token
    pub fn init(&mut self, name: String, symbol: String, decimals: u8, initial_supply: U256) {
        let caller = self.env().caller();
        self.ownable.init(caller);
        self.token.init(symbol, name, decimals, initial_supply);
    }

    delegate! {
        to self.token {
            fn name(&self) -> String;
            fn symbol(&self) -> String;
            fn decimals(&self) -> u8;
            fn total_supply(&self) -> U256;
            fn balance_of(&self, address: &Address) -> U256;
            fn allowance(&self, owner: &Address, spender: &Address) -> U256;
            fn approve(&mut self, spender: &Address, amount: &U256);
            fn decrease_allowance(&mut self, spender: &Address, decr_by: &U256);
            fn increase_allowance(&mut self, spender: &Address, inc_by: &U256);
            fn transfer(&mut self, recipient: &Address, amount: &U256);
            fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256);
        }
    }

    delegate! {
        to self.ownable {
            fn get_owner(&self) -> Address;
            fn transfer_ownership(&mut self, new_owner: &Address);
        }
    }

    /// Mint new tokens (only owner)
    pub fn mint(&mut self, to: &Address, amount: &U256) {
        self.ownable.assert_owner(&self.env().caller());
        self.token.raw_mint(to, amount);
    }

    /// Burn tokens (only token holder)
    pub fn burn(&mut self, amount: &U256) {
        let caller = self.env().caller();
        self.token.raw_burn(&caller, amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::Deployer;

    #[test]
    fn test_sample_token_init() {
        let env = odra_test::env();
        let contract = SampleToken::deploy(
            &env,
            SampleTokenInitArgs {
                name: "Sample Token".to_string(),
                symbol: "ST".to_string(),
                decimals: 18,
                initial_supply: U256::from(1000000u64),
            },
        );

        assert_eq!(contract.symbol(), "ST");
        assert_eq!(contract.name(), "Sample Token");
        assert_eq!(contract.decimals(), 18);
        assert_eq!(contract.total_supply(), U256::from(1000000u64));
    }

    #[test]
    fn test_sample_token_with_different_params() {
        let env = odra_test::env();
        let contract = SampleToken::deploy(
            &env,
            SampleTokenInitArgs {
                name: "Another Token".to_string(),
                symbol: "AT".to_string(),
                decimals: 6,
                initial_supply: U256::from(2000000u64),
            },
        );

        assert_eq!(contract.symbol(), "AT");
        assert_eq!(contract.name(), "Another Token");
        assert_eq!(contract.decimals(), 6);
        assert_eq!(contract.total_supply(), U256::from(2000000u64));
    }

    #[test]
    fn test_mint_tokens() {
        let env = odra_test::env();
        let mut contract = SampleToken::deploy(
            &env,
            SampleTokenInitArgs {
                name: "Sample Token".to_string(),
                symbol: "ST".to_string(),
                decimals: 18,
                initial_supply: U256::from(1000000u64),
            },
        );

        let initial_supply = contract.total_supply();
        let mint_amount = U256::from(100000u64);

        contract.mint(&env.caller(), &mint_amount);

        assert_eq!(contract.total_supply(), initial_supply + mint_amount);
        assert_eq!(
            contract.balance_of(&env.caller()),
            initial_supply + mint_amount
        );
    }
}
