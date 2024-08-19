# Liquidity Pool Implementation

## Overview

This repository contains a Rust-based implementation of a liquidity pool designed for decentralized finance (DeFi) applications. The code simulates the core functionality of a liquidity pool, including initialization, adding and removing liquidity, and swapping tokens. Below is a detailed explanation of each component and method.

## Components

### Initialization

The `LpPool::init` function sets up the liquidity pool with the following parameters:

- `price`: Initial price of the token pair.
- `min_fee`: Minimum fee charged on swaps.
- `max_fee`: Maximum fee charged on swaps.
- `liquidity_target`: Target liquidity level for the pool.

**Validation Checks:**
- `min_fee` must be less than `max_fee`.
- `price` must be greater than 0.
- `liquidity_target` must be greater than 0.

### Adding Liquidity

The `add_liquidity` function allows users to deposit tokens into the pool. It calculates the number of liquidity provider (LP) tokens to mint based on the current value of the pool's assets. This calculation takes into account the proportion of the pool before and after the deposit.

### Removing Liquidity

The `remove_liquidity` function enables users to withdraw their share of the pool. It calculates the amount of tokens and staked tokens to return based on the proportion of LP tokens being redeemed. The function ensures that the requested amount is valid and updates the pool's reserves accordingly.

### Swapping Tokens

The `swap` function allows users to exchange staked tokens for regular tokens. It applies a dynamic fee using the following formula:

```
unstake_fee = max_fee - (max_fee - min_fee) * amount_after / target
```

**Fee Adjustment:**
The fee decreases as the pool's reserves approach the liquidity target, encouraging swaps that help maintain the pool's balance.

### Additional Considerations

The implementation includes various checks and balances to ensure safe operations during adding liquidity, removing liquidity, and swapping tokens. It can be extended or modified to support additional features or specific DeFi use cases.

## Contribution

Contributions are welcome! Please submit issues or pull requests to improve or extend the functionality of the liquidity pool.
