# CasperSwap CLI

A command-line interface tool for deploying and interacting with CasperSwap smart contracts on the Casper Network.

## Overview

CasperSwap is an automated market maker (AMM) DEX implementation based on Uniswap V2, built using the Odra framework. This CLI tool provides easy deployment and management of CasperSwap contracts.

## Installation

From the project root:

```bash
cargo build --release
```

The binary will be available at `target/release/casperswap_cli`.

## Usage

### Help

To see all available commands:

```bash
cargo run --bin casperswap_cli -- --help
```

### Deployment

Deploy all CasperSwap contracts to the network:

```bash
cargo run --bin casperswap_cli -- deploy
```

This will deploy:
- **Factory** contract - manages pair creation and fee collection
- **SampleToken** (instance A) - Test token A (TKNA) with 1 billion initial supply
- **SampleToken** (instance B) - Test token B (TKNB) with 1 billion initial supply
- **CasperswapV2Pair** - Template pair contract

### Contract Interactions

#### Interact with Factory

```bash
cargo run --bin casperswap_cli -- contract Factory <method> [args...]
```

Available methods:
- `fee_to` - Get the current fee collector address
- `set_fee_to <address>` - Set the fee collector address

#### Interact with CasperswapV2Pair

```bash
cargo run --bin casperswap_cli -- contract CasperswapV2Pair <method> [args...]
```

Key methods:
- `initialize <token0> <token1>` - Initialize a pair with two token addresses
- `mint <to>` - Mint liquidity tokens
- `burn <to>` - Burn liquidity tokens
- `swap <amount0Out> <amount1Out> <to> <data>` - Execute a token swap
- `get_reserves` - Get the current reserves

#### Interact with Sample Tokens

The same `SampleToken` contract is deployed twice with different names and symbols.

Interact with SampleTokenA (TKNA):

```bash
cargo run --bin casperswap_cli -- named-contract SampleTokenA <method> [args...]
```

Interact with SampleTokenB (TKNB):

```bash
cargo run --bin casperswap_cli -- named-contract SampleTokenB <method> [args...]
```

Available methods:
- `name` - Get token name
- `symbol` - Get token symbol
- `decimals` - Get token decimals
- `total_supply` - Get total supply
- `balance_of <address>` - Get balance of an address
- `transfer <recipient> <amount>` - Transfer tokens
- `approve <spender> <amount>` - Approve spending
- `mint <to> <amount>` - Mint new tokens (owner only)
- `burn <amount>` - Burn tokens

### Scenarios

Scenarios are high-level operations that combine multiple contract calls.

#### Create a Trading Pair

Initialize a CasperswapV2Pair contract with two token addresses:

```bash
cargo run --bin casperswap_cli -- scenario CreatePair \
  --token0 hash-abc123... \
  --token1 hash-def456...
```

**Note:** The tokens will be automatically sorted (token0 < token1) according to Uniswap V2 convention.

#### Mint Tokens

Mint tokens to a specified address (requires owner privileges):

```bash
cargo run --bin casperswap_cli -- scenario MintTokens \
  --recipient hash-abc123... \
  --amount 1000
```

**Note:** The amount is in base units (tokens) and will be automatically multiplied by 10^18.

## Environment Configuration

The CLI uses the Odra framework's environment configuration. You can configure:

- Network (local, testnet, mainnet)
- RPC endpoint
- Account keys
- Gas limits

Refer to the `Odra.toml` file in the project root for configuration options.

## Examples

### Complete Deployment and Setup

1. Deploy all contracts:
   ```bash
   cargo run --bin casperswap_cli -- deploy
   ```

2. Initialize a pair with the deployed tokens:
   ```bash
   cargo run --bin casperswap_cli -- scenario CreatePair \
     --token0 <token_A_address> \
     --token1 <token_B_address>
   ```

3. Mint tokens to your account:
   ```bash
   cargo run --bin casperswap_cli -- scenario MintTokens \
     --recipient <your_address> \
     --amount 10000
   ```

4. Approve the pair contract to spend your tokens:
   ```bash
   cargo run --bin casperswap_cli -- named-contract SampleTokenA approve \
     <pair_address> 1000000000000000000000
   ```

5. Add liquidity by transferring tokens to the pair and calling mint:
   ```bash
   cargo run --bin casperswap_cli -- named-contract SampleTokenA transfer \
     <pair_address> 100000000000000000000
   
   cargo run --bin casperswap_cli -- named-contract SampleTokenB transfer \
     <pair_address> 100000000000000000000
   
   cargo run --bin casperswap_cli -- contract CasperswapV2Pair mint \
     <your_address>
   ```

## Development

### Adding New Scenarios

To add a new scenario:

1. Create a new file in `cli/scenarios/`
2. Implement the `Scenario` and `ScenarioMetadata` traits
3. Add the scenario to `cli/scenarios/mod.rs`
4. Register it in `cli/cli.rs` using `.scenario(YourScenario)`

Example:

```rust
use odra::host::HostEnv;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, DeployedContractsContainer,
};

pub struct YourScenario;

impl ScenarioMetadata for YourScenario {
    const NAME: &'static str = "YourScenario";
    const DESCRIPTION: &'static str = "Description of your scenario";
}

impl Scenario for YourScenario {
    fn args(&self) -> Vec<CommandArg> {
        // Define your CLI arguments
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args,
    ) -> Result<(), Error> {
        // Implement your scenario logic
        Ok(())
    }
}
```

## Troubleshooting

### Contract Not Found

If you get a "contract not found" error, make sure you've run the deployment first:

```bash
cargo run --bin casperswap_cli -- deploy
```

### Insufficient Gas

If transactions fail due to gas limits, you can adjust the gas in the scenario or contract method implementation.

### Invalid Address Format

Addresses should be in the format `hash-<hex>` or `account-hash-<hex>`.

## License

See the main project LICENSE file.

