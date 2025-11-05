use odra::prelude::OdraError;

#[odra::odra_error]
pub enum RouterError {
    Misconfigured = 21000,
    Expired = 21001,
    InsufficientAAmount = 21002,
    InsufficientBAmount = 21003,
    InsufficientOutputAmount = 21004,
    ExcessiveInputAmount = 21005,
    InvalidPath = 21006,
    PairNotFound = 21007,
    InsufficientBalance = 21008,
}

// Library error types merged from casper_trade_v2_library
#[odra::odra_error]
pub enum LibraryError {
    IdenticalAddresses = 1,
    ZeroAddress = 2,
    InsufficientAmount = 3,
    InsufficientLiquidity = 4,
    InsufficientInputAmount = 5,
    InsufficientOutputAmount = 6,
    InvalidPath = 7,
}
