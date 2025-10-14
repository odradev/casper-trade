# CasperSwap V2 Router Implementation

This document describes the initial router contract stubs created for CasperSwap V2, based on Uniswap V2 Router02.

## Files Created

### 1. `src/casperswap_v2_library.rs`
Library module containing utility functions for router calculations. Based on UniswapV2Library.

**Implemented Functions (Pure/View):**
- `sort_tokens()` - Returns sorted token addresses
- `quote()` - Given an asset amount and pair reserves, returns equivalent amount of other asset
- `get_amount_out()` - Given input amount and reserves, returns maximum output amount (with 0.3% fee)
- `get_amount_in()` - Given output amount and reserves, returns required input amount (with 0.3% fee)
- `get_amounts_out()` - Performs chained getAmountOut calculations for multi-hop swaps
- `get_amounts_in()` - Performs chained getAmountIn calculations for multi-hop swaps

**Stub Functions (To Be Implemented):**
- `pair_for()` - Calculate pair address (needs factory.get_pair() implementation)
- `get_reserves()` - Fetches and sorts reserves for a pair (uses pair_for)

**Error Types:**
- `IdenticalAddresses` - Token addresses are the same
- `ZeroAddress` - One of the tokens is zero address
- `InsufficientAmount` - Amount is zero
- `InsufficientLiquidity` - Reserve is zero
- `InsufficientInputAmount` - Input amount is zero
- `InsufficientOutputAmount` - Output amount is zero
- `InvalidPath` - Path has less than 2 tokens

### 2. `src/casperswap_v2_router.rs`
Main router contract for CasperSwap V2. Based on UniswapV2Router02.

**Implemented Functions:**
- `init()` - Initialize router with factory address
- `factory()` - Returns factory address
- Library wrapper functions (quote, get_amount_out, get_amount_in, get_amounts_out, get_amounts_in)

**Stub Functions (To Be Implemented):**

#### Add Liquidity
- `add_liquidity()` - Add liquidity to a token pair
- `_add_liquidity()` - Internal function to calculate optimal amounts

#### Remove Liquidity
- `remove_liquidity()` - Remove liquidity from a token pair
- `remove_liquidity_with_permit()` - Remove liquidity with gasless approval

#### Swap
- `swap_exact_tokens_for_tokens()` - Swap exact input for tokens
- `swap_tokens_for_exact_tokens()` - Swap tokens for exact output
- `_swap()` - Internal swap function

#### Fee-on-Transfer Token Support
- `swap_exact_tokens_for_tokens_supporting_fee_on_transfer_tokens()` - Swap with fee-on-transfer support
- `_swap_supporting_fee_on_transfer_tokens()` - Internal swap function for fee-on-transfer tokens

**Error Types:**
- `Expired` - Transaction deadline has passed
- `InsufficientAAmount` - Insufficient token A amount
- `InsufficientBAmount` - Insufficient token B amount
- `InsufficientOutputAmount` - Insufficient output amount
- `ExcessiveInputAmount` - Excessive input amount
- `InvalidPath` - Invalid swap path

## Notable Differences from Uniswap V2

1. **No ETH/WETH Functions**: Since Casper blockchain doesn't use ETH, we've excluded:
   - `addLiquidityETH()`
   - `removeLiquidityETH()`
   - `swapExactETHForTokens()`
   - `swapTokensForExactETH()`
   - `swapExactTokensForETH()`
   - `swapETHForExactTokens()`
   
   These can be added later with WCSPR (Wrapped CSPR) support if needed.

2. **Pair Address Calculation**: Uniswap uses CREATE2 to calculate pair addresses deterministically. In Casper, we need to call the factory's `get_pair()` method instead.

3. **ContractRef Pattern**: Uses Odra's ContractRef pattern for cross-contract calls instead of Solidity's interface pattern.

## Next Steps

To complete the router implementation:

1. **Implement Factory's `get_pair()` method**
   - Add mapping to store pair addresses
   - Update `pair_for()` in library to call factory

2. **Implement Add Liquidity**
   - `_add_liquidity()` - Calculate optimal amounts based on reserves
   - `add_liquidity()` - Transfer tokens and mint LP tokens

3. **Implement Remove Liquidity**
   - Transfer LP tokens to pair
   - Call pair's `burn()` function
   - Verify minimum amounts received

4. **Implement Swap Functions**
   - `_swap()` - Loop through path and perform swaps
   - `swap_exact_tokens_for_tokens()` - Calculate amounts and execute swap
   - `swap_tokens_for_exact_tokens()` - Calculate amounts and execute swap

5. **Implement Fee-on-Transfer Support**
   - Handle tokens that charge fees on transfer
   - Check actual balances instead of calculated amounts

6. **Add Deadline Checks**
   - Implement deadline modifier/check in all public functions

7. **Add Comprehensive Tests**
   - Test library functions with various inputs
   - Test add/remove liquidity scenarios
   - Test single and multi-hop swaps
   - Test edge cases and error conditions

## Reference Implementation

The original Uniswap V2 Router implementation can be found in:
- `/home/kuba/Projekty/v2-periphery/contracts/UniswapV2Router02.sol`
- `/home/kuba/Projekty/v2-periphery/contracts/libraries/UniswapV2Library.sol`

Tests are available in:
- `/home/kuba/Projekty/v2-periphery/test/UniswapV2Router02.spec.ts`

