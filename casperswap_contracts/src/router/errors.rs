use odra::prelude::OdraError;

#[odra::odra_error]
    pub enum CasperswapV2RouterError {
        Misconfigured = 21000,
        Expired = 21001,
        InsufficientAAmount = 21002,
        InsufficientBAmount = 21003,
        InsufficientOutputAmount = 21004,
        ExcessiveInputAmount = 21005,
        InvalidPath = 21006,
    }