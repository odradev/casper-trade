use odra::prelude::OdraError;

#[odra::odra_error]
pub enum CasperswapV2PairError {
    /// The pair is not properly configured.
    Misconfigured = 20001,
    /// The caller is not allowed to perform this action.
    Forbidden = 20002,
    /// The pair is not properly initialized.
    NotInitialized = 20003,
    /// Insufficient liquidity minted.
    InsufficientLiquidityMinted = 20004,
}
