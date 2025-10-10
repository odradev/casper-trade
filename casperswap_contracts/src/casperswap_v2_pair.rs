use odra::prelude::*;
pub mod events;

pub const MINIMUM_LIQUIDITY: u64 = 1000;

/// CasperswapV2Pair contract - implementation based on Uniswap V2
#[odra::module]
pub struct CasperswapV2Pair {}

/// Module implementation
#[odra::module]
impl CasperswapV2Pair {}

impl CasperswapV2Pair {}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::Deployer;

    #[test]
    fn test_pair_init() {
        let env = odra_test::env();
    }
}
