# Casper Trade

A decentralized exchange (DEX) implementation based on Uniswap V2, built for the Casper Network using the Odra framework. Casper Trade provides automated market maker (AMM) functionality with liquidity pools, token swaps, and yield farming capabilities.

## Features

- **Automated Market Maker (AMM)**: Decentralized token exchange with liquidity pools
- **Factory Contract**: Manages pair creation and fee collection
- **Sample Tokens**: CEP-18 compatible tokens for testing and development
- **CLI Tool**: Command-line interface for deployment and interaction
- **Casper Network Integration**: Built specifically for Casper blockchain

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [cargo-odra](https://github.com/odradev/cargo-odra) - Odra framework CLI tool
- [Docker](https://www.docker.com/) (for local Casper node testing)

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

This project uses [just](https://github.com/casey/just) for task automation. Install just if you haven't already:

```bash
# On macOS
brew install just

# On Ubuntu/Debian
sudo apt install just

# On Arch Linux
sudo pacman -S just
```

### Available Commands

#### Testing
```bash
# Run all tests (OdraVM and Casper backend)
just test

# Run tests on OdraVM only
cargo odra test

# Run tests on Casper backend
cargo odra test -b casper
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

The compiled WASM files will be placed in the `wasm/` directory:
- `CasperTradeV2Pair.wasm` - Main pair contract
- `Factory.wasm` - Factory contract for pair management
- `SampleToken.wasm` - Sample CEP-18 token contract

## Testing

### Local Testing (OdraVM)
```bash
cargo odra test
```

### Casper Backend Testing
```bash
cargo odra test -b casper
```

### All Tests
```bash
just test
```

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

## Local Development

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

- [Odra Framework](https://odra.dev/)
- [Casper Network](https://casper.network/)
- [Uniswap V2](https://uniswap.org/docs/v2/)
- [CEP-18 Standard](https://github.com/casper-ecosystem/cep-18)