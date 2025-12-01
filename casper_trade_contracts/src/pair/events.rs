use odra::casper_types::U512;
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

#[odra::event]
pub struct FeeToUpdated {
    pub old: Option<Address>,
    pub new: Option<Address>,
}

#[odra::event]
pub struct FactoryInitialized {
    pub fee_to: Option<Address>,
    pub pair_factory: Address,
}

#[odra::event]
pub struct PairInitialized {
    pub token0: Address,
    pub token1: Address,
}

#[odra::event]
pub struct SkimExcess {
    pub to: Address,
    pub amount0: U256,
    pub amount1: U256,
}

#[odra::event]
pub struct ProtocolFeeMinted {
    pub to: Address,
    pub liquidity: U256,
}

#[odra::event]
pub struct CSPRRefunded {
    pub to: Address,
    pub amount: U512,
}
