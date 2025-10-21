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

#### Deploy to Local Node (NCTL)

For local development and testing using nctl:

```bash
# First, ensure nctl container is running
docker ps | grep nctl

# Clear any cached testnet contracts (important!)
rm -f resources/contracts.toml

# Deploy all contracts
just cli-on-nctl deploy
```

This will deploy:
- **Factory** contract - manages pair creation and fee collection
- **SampleTokenA** - Test token A (TKNA) with 1 billion initial supply
- **SampleTokenB** - Test token B (TKNB) with 1 billion initial supply
- **WrappedNativeToken** - Wrapped CSPR token (WCSPR)
- **CasperswapV2Router** - Router for trading operations
- **TokenA_TokenB** - Trading pair for TKNA/TKNB (pre-initialized)
- **TokenA_WCSPR** - Trading pair for TKNA/WCSPR (pre-initialized)
- **TokenB_WCSPR** - Trading pair for TKNB/WCSPR (pre-initialized)

#### Deploy to Testnet

```bash
# Clear any cached local contracts (important!)
rm -f resources/contracts.toml

# Deploy all contracts
cargo run --bin casperswap_cli -- deploy
```

**Important:** The CLI caches deployed contract addresses in `resources/contracts.toml`. When switching between environments (testnet ↔ nctl), always clear this file first to avoid address conflicts.

### Quick Start: Add Liquidity

After deployment, you can immediately add liquidity to any of the pre-deployed pairs:

#### Add Liquidity with CSPR

```bash
# Add 100 TKNA + 50 CSPR to TokenA-WCSPR pair
just cli-on-nctl scenario AddLiquidityCSPR \
  --token_a SampleTokenA \
  --amount_a 100 \
  --amount_cspr 50
```

#### Add Liquidity with Two Tokens

```bash
# Add 100 TKNA + 200 TKNB to TokenA-TokenB pair
just cli-on-nctl scenario AddLiquidity \
  --token_a SampleTokenA \
  --token_b SampleTokenB \
  --amount_a 100 \
  --amount_b 200
```

**Note:** The CLI uses the deployer's account, which already has the initial token supply (1 billion tokens of each).

### Complete Workflow Example

Here's a complete workflow for deploying and using CasperSwap on local nctl:

```bash
# 1. Clear cache and deploy
rm -f resources/contracts.toml
just cli-on-nctl deploy

# 2. Add liquidity to TokenA-WCSPR pair
just cli-on-nctl scenario AddLiquidityCSPR \
  --token_a SampleTokenA \
  --amount_a 1000 \
  --amount_cspr 500

# 3. Add liquidity to TokenA-TokenB pair
just cli-on-nctl scenario AddLiquidity \
  --token_a SampleTokenA \
  --token_b SampleTokenB \
  --amount_a 1000 \
  --amount_b 1000

# 4. Swap tokens (example)
just cli-on-nctl scenario SwapTokens \
  --token_in SampleTokenA \
  --token_out SampleTokenB \
  --amount_in 10
```

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

#### Setup a Trading Pair

Create, initialize, and register a trading pair with the factory. Note that three pairs are already deployed and configured during deployment (TokenA_TokenB, TokenA_WCSPR, TokenB_WCSPR). Use this scenario only if you need to create additional custom pairs:

```bash
cargo run --bin casperswap_cli -- scenario SetupPair \
  --token_a SampleTokenA \
  --token_b hash-abc123...
```

**Note:** You can use either contract names (like "SampleTokenA") or addresses (like "hash-..."). The tokens will be automatically sorted according to Uniswap V2 convention, and the pair will be registered with the factory.

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

See the [Complete Workflow Example](#complete-workflow-example) section above for a step-by-step guide.

### Mint Tokens to Another Account

If you need to mint tokens to another account:
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

### "Couldn't query for entity address value" Error

This error occurs when the CLI tries to use cached contract addresses from a different environment.

**Problem:** You deployed to testnet, and now trying to use nctl (or vice versa). The CLI is reading cached addresses from `resources/contracts.toml` that don't exist in the current environment.

**Solution:** Clear the cache before switching environments:
```bash
rm -f resources/contracts.toml
```

Then deploy again to the current environment.

**Why this happens:** The CLI caches deployed contract addresses to avoid unnecessary redeployments. However, testnet and local nctl use different blockchain networks with different addresses.

**Best Practice:** Always clear `resources/contracts.toml` when switching between:
- Testnet ↔ Local NCTL
- Different nctl instances (after restart)
- Different testnets

### Contract Not Found

If you get a "contract not found" error, make sure you've run the deployment first:

```bash
# For local nctl
just cli-on-nctl deploy

# For testnet
cargo run --bin casperswap_cli -- deploy
```

### Insufficient Gas

If transactions fail due to gas limits, you can adjust the gas in the scenario or contract method implementation.

### Invalid Address Format

Addresses should be in the format `hash-<hex>` or `account-hash-<hex>`.

### Approval/Transfer Errors (User error: 60002)

If you get error 60002 when adding liquidity, it usually means the router doesn't have approval to spend your tokens. The AddLiquidity scenarios should handle this automatically, but if you're calling contracts directly, remember to approve first:

```bash
just cli-on-nctl named-contract SampleTokenA approve \
  <router_address> <amount>
```

## License

See the main project LICENSE file.

