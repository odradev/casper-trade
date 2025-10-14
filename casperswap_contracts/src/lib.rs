#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
extern crate alloc;

pub mod casperswap_callee;
pub mod casperswap_v2_pair;
pub mod router;
pub mod factory;
pub mod sample_tokens;
pub mod utils;