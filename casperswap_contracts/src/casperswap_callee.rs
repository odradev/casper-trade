use odra::{
    casper_types::{bytesrepr::Bytes, U256},
    prelude::*,
};

#[odra::external_contract]
pub trait CasperswapCallee {
    fn casperswap_call(&self, sender: Address, amount0: U256, amount1: U256, data: Bytes);
}
