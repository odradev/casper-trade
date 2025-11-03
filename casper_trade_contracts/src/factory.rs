use crate::casper_trade_v2_pair::{
    CasperTradeV2Pair, CasperTradeV2PairContractRef, CasperTradeV2PairFactoryContractRef,
};
use crate::router::errors::CasperTradeV2RouterError::Misconfigured;
use odra::prelude::*;
use odra::ContractRef;

#[odra::module]
pub struct Factory {
    fee_to: Var<Option<Address>>,
    pairs: Mapping<(Address, Address), Address>,
    pair_factory: Var<Address>,
}

#[odra::module]
impl Factory {
    /// Initializes the factory with the given fee to address.
    ///
    /// If fee to is None, the factory will not charge any fees.
    pub fn init(&mut self, fee_to: Option<Address>, pair_factory: Address) {
        self.fee_to.set(fee_to);
        self.pair_factory.set(pair_factory);
    }

    pub fn fee_to(&self) -> Option<Address> {
        self.fee_to.get().unwrap_or_revert(self)
    }

    pub fn set_fee_to(&mut self, fee_to: Option<Address>) {
        self.fee_to.set(fee_to);
    }

    /// Creates a pair for the given tokens.ists, it will return existing one.
    /// In the mock implementation, this looks up the pre-configured pair and stores it.
    pub fn create_pair(&mut self, token_a: Address, token_b: Address) -> Address {
        let (token0, token1) = self.sort_tokens(token_a, token_b);
        match self.pairs.get(&(token0, token1)) {
            None => {
                let mut contract_factory = CasperTradeV2PairFactoryContractRef::new(
                    self.env(),
                    self.pair_factory.get_or_revert_with(Misconfigured),
                );
                let pair =
                    contract_factory.factory("LatestPair".to_string(), self.env().self_address());
                let mut pair_instance = CasperTradeV2PairContractRef::new(self.env(), pair.0);
                pair_instance.initialize(token0, token1);
                self.pairs.set(&(token0, token1), pair.0);
                pair.0
            }
            Some(pair) => pair,
        }
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
