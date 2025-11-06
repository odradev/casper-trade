# Casper Trade

A decentralized exchange (DEX) implementation based on Uniswap V2,
built for the Casper Network using the Odra framework. 
Casper Trade provides automated market maker (AMM) functionality
with liquidity pools, token swaps, and yield farming capabilities.

## Architecture
The implementation tries to follow the Uniswap V2 implementation 1 to 1.
It is based on the two following repositories:
 
- https://github.com/Uniswap/v2-periphery
- https://github.com/Uniswap/v2-core

Due to differences between Casper and EVM, there are some minor
differences in the architecture:

### Factory
Uniswap's version of the factory deploys Pair contracts directly using
`create2` and Pair's bytecode stored in the Factory contract.

Casper's mechanism is different and Odra's implementation of it is as follows:
- Odra automatically generates PairFactory contract for the Pair (`factory=on` directive)
- PairFactory's sole purpose is to manage deployments and upgrades of new Pair contracts
- Factory's `create_pair` calls `factory()` method of PairFactory contract. PairFactory
deploys a new Pair contract and returns the address of the new contract to the Factory.

### Tests
Tests are based on the original Uniswap tests to ensure parity.
Router tests are based on UniswapV2Router01.spec.ts and UniswapV2Router02.spec.ts
from `v2-periphery` repository.

Factory tests are based on UniswapV2Factory.spec.ts from `v2-core` repository.

Pair tests are based on UniswapV2Pair.spec.ts from `v2-core` repository.

### Contracts
Below is the list of contracts created and used by the project.

- `Pair` - Main pair contract. It will not be deployed directly, instead PairFactory will deploy it in runtime.
It corresponds to UniswapV2Pair contract from Uniswap.
- `PairFactory` - Pair contract factory. Its code is generated automatically by Odra. 
It is responsible for deploying Pair contracts.
- `Router` - Router contract. It corresponds to UniswapV2Router.
- `Factory` - Factory contract for pair management. Its corresponding Uniswap contract is UniswapV2Factory.
- `SampleToken` - Sample CEP-18 token contract used for testing.
- `WrappedNativeToken` - Token from Odra modules used for testing.


## Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [cargo-odra](https://github.com/odradev/cargo-odra) - Odra framework CLI tool
- [Docker](https://www.docker.com/) (optional - for local Casper node testing)

## Installation

1. Clone the repository:
```bash
git clone git@github.com:odradev/casper-trade.git
cd casper-trade
```

2. Install cargo-odra:
```bash
cargo install cargo-odra
```

3. Build the project:
```bash
cargo odra build
```

## Quick Start

### Using Just Commands

This project uses [just](https://github.com/casey/just) for task automation.

### Available Commands

#### Testing
```bash
# Run all tests (Casper backend)
just test
```

#### Code Quality
```bash
# Run clippy linter
just clippy

# Format code
just lint

# Check formatting without changing files
just check-lint
```

#### CLI Usage
```bash
# Run CLI with arguments
just cli <arguments>

# Example: Deploy contracts
just cli deploy

# Example: Get help
just cli --help
```

#### Local Casper Node Testing
```bash
# Start local Casper node with nctl
just run-nctl

# Run CLI commands against local node
just cli-on-nctl <arguments>

# Example: Deploy to local node
just cli-on-nctl deploy
```

## Building

### Casper Backend Build
To build WASM files for Casper Network deployment:
```bash
cargo odra build -b casper
```

The compiled WASM files will be placed in the `wasm/` directory.

## CLI Tool

The Casper Trade CLI provides easy deployment and interaction with contracts. See [CLI documentation](cli/README.md) for detailed usage.

### Basic Usage
```bash
# Deploy all contracts
just cli deploy

# Create a trading pair
just cli scenario CreatePair --token0 <address> --token1 <address>

# Mint tokens
just cli scenario MintTokens --recipient <address> --amount 1000
```

## Local casper node

### Setting up Local Casper Node

1. Start the local Casper node:
```bash
just run-nctl
```

2. Deploy contracts to local node:
```bash
just cli-on-nctl deploy
```

3. Interact with deployed contracts:
```bash
just cli-on-nctl contract Factory fee_to
```

## Resources

- [Odra Framework](https://odra.dev/docs)
- [Casper Network](https://casper.network/)
- [Uniswap V2](https://uniswap.org/docs/v2/)
- [CEP-18 Standard](https://github.com/casper-ecosystem/cep-18)