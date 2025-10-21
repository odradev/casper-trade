use odra::{casper_types::U256, prelude::*};

#[odra::event]
pub struct Approval {
    pub owner: Address,
    pub spender: Address,
    pub value: U256,
}

#[odra::event]
pub struct Transfer {
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub value: U256,
}

#[odra::event]
pub struct Mint {
    pub sender: Address,
    pub amount0: U256,
    pub amount1: U256,
}

#[odra::event]
pub struct Burn {
    pub sender: Address,
    pub amount0: U256,
    pub amount1: U256,
    pub to: Address,
}

#[odra::event]
pub struct Swap {
    pub sender: Address,
    pub amount0_in: U256,
    pub amount1_in: U256,
    pub amount0_out: U256,
    pub amount1_out: U256,
    pub to: Address,
}

#[odra::event]
pub struct Sync {
    pub reserve0: U256,
    pub reserve1: U256,
}
