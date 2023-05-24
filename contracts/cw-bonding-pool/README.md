# cw-bonding-pool


## Features 
- [X] **SetActive**: This action sets whether the pool is active. In the context of a bonding curve, the `is_active` parameter could be used to indicate whether the bonding curve is currently allowing transactions or not.

- [X] **SwapExactAmountIn**: This action needs to be modified to calculate the number of tokens out based on the bonding curve formula. The transaction should still revert if the minimum amount of tokens out is not received.

- [X] **SwapExactAmountOut**: This action needs to be modified to calculate the number of tokens in based on the bonding curve formula. The transaction should still revert if the maximum amount of tokens in is exceeded.

- [X] **GetSwapFee**: In a bonding curve model, the swap fee might be replaced by a slippage parameter. This action could be renamed to `GetSlippage` and should return the current slippage setting for the bonding curve.

- [X] **IsActive**: This action remains the same, returning the current active status of the bonding curve.

- [X] **GetTotalPoolLiquidity**: This action remains the same, returning the total liquidity in the bonding curve.

- [X] **SpotPrice**: This action needs to be modified to calculate the spot price based on the bonding curve formula using the current state of the bonding curve.

- [X] **CalcOutAmtGivenIn**: This action needs to be modified to calculate the number of tokens out given the number of tokens in based on the bonding curve formula. This will likely involve integrating the bonding curve formula from the current point to the point after the tokens in are added.

- [X] **CalcInAmtGivenOut**: This action needs to be modified to calculate the number of tokens in given the number of tokens out based on the bonding curve formula. Like `CalcOutAmtGivenIn`, this will likely involve integrating the bonding curve formula.
