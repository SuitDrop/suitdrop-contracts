use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal, Uint128};

use crate::{
    curves::{
        decimal, Constant, CubeRootSquared, Curve, DecimalPlaces, Linear, SquareRoot,
        SquareRootCubed,
    },
    state::CurveState,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub supply_subdenom: String,
    /// number of decimal places of the supply token, needed for proper curve math.
    /// If it is eg. BTC, where a balance of 10^8 means 1 BTC, then use 8 here.
    pub supply_decimals: u8,

    // maximum supply of the token
    pub max_supply: Uint128,

    /// this is the reserve token denom (only support native for now)
    pub reserve_denom: String,
    /// number of decimal places for the reserve token, needed for proper curve math.
    /// Same format as decimals above, eg. if it is uatom, where 1 unit is 10^-6 ATOM, use 6 here
    pub reserve_decimals: u8,

    /// enum to store the curve parameters used for this contract
    /// if you want to add a custom Curve, you should make a new contract that imports this one.
    /// write a custom `instantiate`, and then dispatch `your::execute` -> `cw20_bonding::do_execute`
    /// with your custom curve as a parameter (and same with `query` -> `do_query`)
    pub curve_type: CurveType,

    /// Enable if you want to test the contract without cosmwasmpool
    pub test_mode: Option<bool>,

    // Enable if you want to simulate the contract off-chain
    pub simulation_mode: Option<bool>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Dissolve {},
    Sudo(SudoMsg),
    Simulate(SimulationMsg),
}

#[cw_serde]
pub enum SimulationMsg {
    SetState { state: BondingPoolState },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// GetSwapFee returns the pool's swap fee, based on the current state.
    /// Pools may choose to make their swap fees dependent upon state
    /// (prior TWAPs, network downtime, other pool states, etc.)
    /// This is intended to be fee that is collected by liquidity providers.
    /// If the contract provider wants to collect fee for itself, it should implement its own fee collection mechanism.
    #[returns(GetSwapFeeResponse)]
    GetSwapFee {},

    /// Returns whether the pool has swaps enabled at the moment
    #[returns(IsActiveResponse)]
    IsActive {},

    /// GetTotalShares returns the total number of LP shares in the pool

    /// GetTotalPoolLiquidity returns the coins in the pool owned by all LPs
    #[returns(TotalPoolLiquidityResponse)]
    GetTotalPoolLiquidity {},

    /// Returns the spot price of the 'base asset' in terms of the 'quote asset' in the pool,
    /// errors if either baseAssetDenom, or quoteAssetDenom does not exist.
    /// For example, if this was a UniV2 50-50 pool, with 2 ETH, and 8000 UST
    /// pool.SpotPrice(ctx, "eth", "ust") = 4000.00
    #[returns(SpotPriceResponse)]
    SpotPrice {
        quote_asset_denom: String,
        base_asset_denom: String,
    },

    /// CalcOutAmtGivenIn calculates the amount of tokenOut given tokenIn and the pool's current state.
    /// Returns error if the given pool is not a CFMM pool. Returns error on internal calculations.
    #[returns(CalcOutAmtGivenInResponse)]
    CalcOutAmtGivenIn {
        token_in: Coin,
        token_out_denom: String,
        swap_fee: Decimal,
    },

    /// CalcInAmtGivenOut calculates the amount of tokenIn given tokenOut and the pool's current state.
    /// Returns error if the given pool is not a CFMM pool. Returns error on internal calculations.
    #[returns(CalcInAmtGivenOutResponse)]
    CalcInAmtGivenOut {
        token_out: Coin,
        token_in_denom: String,
        swap_fee: Decimal,
    },

    // Non cosmwasmpool queries
    #[returns(BondingPoolState)]
    BondingPoolState {},
}

#[cw_serde]
pub struct BondingPoolState {
    pub curve_state: CurveState,
    pub dissolved_curve_state: CurveState,
    pub curve_type: CurveType,
    pub is_active: bool,
}

#[cw_serde]
pub struct GetSwapFeeResponse {
    pub swap_fee: Decimal,
}

#[cw_serde]
pub struct IsActiveResponse {
    pub is_active: bool,
}

#[cw_serde]
pub struct TotalPoolLiquidityResponse {
    pub total_pool_liquidity: Vec<Coin>,
}

#[cw_serde]
pub struct SpotPriceResponse {
    pub spot_price: Decimal,
}

#[cw_serde]
pub struct CalcOutAmtGivenInResponse {
    pub token_out: Coin,
}

#[cw_serde]
pub struct CalcInAmtGivenOutResponse {
    pub token_in: Coin,
}

#[cw_serde]
pub struct GetSharesResponse {
    pub shares: Uint128,
}

#[cw_serde]
pub struct GetTotalSharesResponse {
    pub total_shares: Uint128,
}

#[cw_serde]
pub struct GetTotalPoolLiquidityResponse {
    pub total_pool_liquidity: Vec<Coin>,
}

/// ------------------  ------------------
/// SUDO MSGS
/// ------------------  ------------------
///

#[cw_serde]
pub enum SudoMsg {
    /// SetActive sets the active status of the pool.
    SetActive { is_active: bool },
    /// SwapExactAmountIn swaps an exact amount of tokens in for as many tokens out as possible.
    /// The amount of tokens out is determined by the current exchange rate and the swap fee.
    /// The user specifies a minimum amount of tokens out, and the transaction will revert if that amount of tokens
    /// is not received.
    SwapExactAmountIn {
        sender: String,
        token_in: Coin,
        token_out_denom: String,
        token_out_min_amount: Uint128,
        swap_fee: Decimal,
    },
    /// SwapExactAmountOut swaps as many tokens in as possible for an exact amount of tokens out.
    /// The amount of tokens in is determined by the current exchange rate and the swap fee.
    /// The user specifies a maximum amount of tokens in, and the transaction will revert if that amount of tokens
    /// is exceeded.
    SwapExactAmountOut {
        sender: String,
        token_in_denom: String,
        token_in_max_amount: Uint128,
        token_out: Coin,
        swap_fee: Decimal,
    },
}

/// ------------------  ------------------
/// EXECUTE MSGS
/// ------------------  ------------------
///

#[cw_serde]
pub enum SudoMessage {
    /// SwapExactAmountIn swaps an exact amount of tokens in for as many tokens out as possible.
    /// The amount of tokens out is determined by the current exchange rate and the swap fee.
    /// The user specifies a minimum amount of tokens out, and the transaction will revert if that amount of tokens
    /// is not received.
    SwapExactAmountIn {
        sender: String,
        token_in: Coin,
        token_out_denom: String,
        token_out_min_amount: Uint128,
        swap_fee: Decimal,
    },
    /// SwapExactAmountOut swaps as many tokens in as possible for an exact amount of tokens out.
    /// The amount of tokens in is determined by the current exchange rate and the swap fee.
    /// The user specifies a maximum amount of tokens in, and the transaction will revert if that amount of tokens
    /// is exceeded.
    SwapExactAmountOut {
        sender: String,
        token_in_denom: String,
        token_in_max_amount: Uint128,
        token_out: Coin,
        swap_fee: Decimal,
    },
}

#[cw_serde]
/// Fixing token in amount makes token amount out varies
pub struct SwapExactAmountInResponseData {
    pub token_out_amount: Uint128,
}

#[cw_serde]
/// Fixing token out amount makes token amount in varies
pub struct SwapExactAmountOutResponseData {
    pub token_in_amount: Uint128,
}

#[cw_serde]
pub enum MigrateMsg {}

pub type CurveFn = Box<dyn Fn(DecimalPlaces) -> Box<dyn Curve>>;

#[cw_serde]
pub enum CurveType {
    /// Constant always returns `value * 10^-scale` as spot price
    Constant { value: Uint128, scale: u32 },
    /// Linear returns `slope * 10^-scale * supply` as spot price
    Linear { slope: Uint128, scale: u32 },
    /// SquareRoot returns `slope * 10^-scale * supply^0.5` as spot price
    SquareRoot { slope: Uint128, scale: u32 },
    /// SquareRootCubed returns `f(x) = slope * ((x * 10^-scale)^(1/2))^3`
    SquareRootCubed { slope: Uint128, scale: u32 },
    /// CubeRootSquared returns `f(x) = slope * ((x * 10^-scale)^(1/3))^2`
    CubeRootSquared { slope: Uint128, scale: u32 },
}

impl CurveType {
    pub fn to_curve_fn(&self) -> CurveFn {
        match self.clone() {
            CurveType::Constant { value, scale } => {
                let calc = move |places| -> Box<dyn Curve> {
                    Box::new(Constant::new(decimal(value, scale), places))
                };
                Box::new(calc)
            }
            CurveType::Linear { slope, scale } => {
                let calc = move |places| -> Box<dyn Curve> {
                    Box::new(Linear::new(decimal(slope, scale), places))
                };
                Box::new(calc)
            }
            CurveType::SquareRoot { slope, scale } => {
                let calc = move |places| -> Box<dyn Curve> {
                    Box::new(SquareRoot::new(decimal(slope, scale), places))
                };
                Box::new(calc)
            }
            CurveType::SquareRootCubed { slope, scale } => {
                let calc = move |places| -> Box<dyn Curve> {
                    Box::new(SquareRootCubed::new(decimal(slope, scale), places))
                };
                Box::new(calc)
            }
            CurveType::CubeRootSquared { slope, scale } => {
                let calc = move |places| -> Box<dyn Curve> {
                    Box::new(CubeRootSquared::new(decimal(slope, scale), places))
                };
                Box::new(calc)
            }
        }
    }
}
