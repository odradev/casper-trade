use odra::{casper_types::U256, prelude::*};

#[odra::event]
pub struct PairMint {
    pub sender: Address,
    pub amount0: U256,
    pub amount1: U256,
}

#[odra::event]
pub struct PairBurn {
    pub sender: Address,
    pub amount0: U256,
    pub amount1: U256,
    pub to: Address,
}

#[odra::event]
pub struct PairSwap {
    pub sender: Address,
    pub amount0_in: U256,
    pub amount1_in: U256,
    pub amount0_out: U256,
    pub amount1_out: U256,
    pub to: Address,
}

#[odra::event]
pub struct PairSync {
    pub reserve0: U256,
    pub reserve1: U256,
}
