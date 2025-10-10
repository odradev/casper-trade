use odra::casper_types::U256;
use odra::prelude::*;
use odra_modules::cep18_token::Cep18;

pub const MINIMUM_LIQUIDITY: u64 = 1000;

/// CasperswapV2Pair contract - AMM implementation based on Uniswap V2
#[odra::module]
pub struct CasperswapV2Pair {
}

/// Module implementation
#[odra::module]
impl CasperswapV2Pair {
}

impl CasperswapV2Pair {
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::Deployer;


    #[test]
    fn test_pair_init() {
        let env = odra_test::env();
    }
}
