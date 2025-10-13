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
    /// Arithmetic overflow occurred.
    Overflow = 20005,
    /// Insufficient output amount.
    InsufficientOutputAmount = 20006,
    /// Insufficient liquidity.
    InsufficientLiquidity = 20007,
    /// Invalid recipient address.
    InvalidTo = 20008,
    /// Insufficient input amount.
    InsufficientInputAmount = 20009,
    /// K invariant check failed.
    K = 20010,
}
