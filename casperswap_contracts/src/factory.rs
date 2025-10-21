use odra::prelude::*;

#[odra::module]
pub struct Factory {
    fee_to: Var<Option<Address>>,
    pairs: Mapping<(Address, Address), Address>,
    mock_pairs: Mapping<(Address, Address), Address>,
}

#[odra::module]
impl Factory {
    /// Initializes the factory with the given fee to address.
    ///
    /// If fee to is None, the factory will not charge any fees.
    pub fn init(&mut self, fee_to: Option<Address>) {
        self.fee_to.set(fee_to);
    }

    pub fn fee_to(&self) -> Option<Address> {
        self.fee_to.get().unwrap_or_revert(self)
    }

    pub fn set_fee_to(&mut self, fee_to: Option<Address>) {
        self.fee_to.set(fee_to);
    }

    /// Sets up a mock pair mapping for testing.
    /// Immediately stores the pair in the pairs mapping so it's available for get_pair.
    /// Also sets up the mock so create_pair returns the same pair.
    pub fn will_create_pair(&mut self, token_a: Address, token_b: Address, pair: Address) {
        let (token0, token1) = self.sort_tokens(token_a, token_b);
        self.mock_pairs.set(&(token0, token1), pair);
        self.pairs.set(&(token0, token1), pair);
    }

    /// Creates a pair for the given tokens.
    /// In the mock implementation, this looks up the pre-configured pair and stores it.
    pub fn create_pair(&mut self, token_a: Address, token_b: Address) -> Address {
        let (token0, token1) = self.sort_tokens(token_a, token_b);
        let pair = self.mock_pairs
            .get(&(token0, token1))
            .unwrap_or_revert_with(self, errors::FactoryError::CreatingAPairWithoutMockingIt);
        self.pairs.set(&(token0, token1), pair);
        pair
    }

    /// Returns the pair address for the given tokens, if it exists.
    pub fn get_pair(&self, token_a: Address, token_b: Address) -> Option<Address> {
        let (token0, token1) = self.sort_tokens(token_a, token_b);
        self.pairs.get(&(token0, token1))
    }

    /// Sorts two token addresses to ensure consistent ordering.
    fn sort_tokens(&self, token_a: Address, token_b: Address) -> (Address, Address) {
        if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        }
    }
}

pub mod errors {
    use odra::prelude::*;

    #[odra::odra_error]
    pub enum FactoryError {
        CreatingAPairWithoutMockingIt = 1,
    }
}
