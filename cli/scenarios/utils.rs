use casper_trade_contracts::sample_tokens::SampleToken;
use odra::host::{HostEnv, HostRef};
use odra::prelude::{Address, Addressable};
use odra_cli::{scenario::Error, ContractProvider, DeployedContractsContainer};
use odra_modules::cep18_token::Cep18HostRef;

/// Parses a token input that can be either a contract name or an address.
/// Returns the address and a display name for the token.
///
/// # Arguments
/// * `input` - The token identifier (contract name like "SampleTokenA" or address like "hash-...")
/// * `param_name` - The name of the parameter (for error messages)
/// * `env` - The host environment
/// * `container` - The deployed contracts container
///
/// # Returns
/// * `Ok((address, display_name))` - The token address and a user-friendly display name
/// * `Err(Error)` - If the input is neither a valid address nor a known contract name
pub fn parse_token_input(
    input: &str,
    param_name: &str,
    env: &HostEnv,
    container: &DeployedContractsContainer,
) -> Result<(Address, String), Error> {
    // Try to parse as an address first
    if let Ok(addr) = input.parse::<Address>() {
        // Successfully parsed as address - get token symbol for display
        let token = Cep18HostRef::new(addr, env.clone());
        let display_name = format!("{} ({})", token.symbol(), input);
        Ok((addr, display_name))
    } else {
        // Try as contract name
        match container.contract_ref_named::<SampleToken>(env, Some(input.to_string())) {
            Ok(token) => {
                let addr = token.address();
                let display_name = format!("{} ({})", token.symbol(), input);
                Ok((addr, display_name))
            }
            Err(_) => Err(Error::OdraError {
                message: format!(
                    "Invalid {}: '{}' is neither a valid address nor a known contract name",
                    param_name, input
                ),
            }),
        }
    }
}

/// Creates a CEP-18 token reference from an address
pub fn create_token_ref(address: Address, env: &HostEnv) -> Cep18HostRef {
    Cep18HostRef::new(address, env.clone())
}
