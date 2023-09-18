use cosmwasm_schema::cw_serde;

use cosmwasm_std::Uint128;
use cw_storage_plus::Item;

use crate::curves::DecimalPlaces;
use crate::msg::CurveType;

/// Supply is dynamic and tracks the current supply of staked and ERC20 tokens.
#[cw_serde]
pub struct CurveState {
    /// reserve is how many native tokens exist bonded to the validator
    pub reserve: Uint128,
    /// supply is how many tokens this contract has issued
    pub supply: Uint128,

    pub supply_denom: String,

    // the denom of the reserve token
    pub reserve_denom: String,

    // how to normalize reserve and supply
    pub decimals: DecimalPlaces,
}

impl CurveState {
    pub fn new(reserve_denom: String, supply_denom: String, decimals: DecimalPlaces) -> Self {
        CurveState {
            reserve: Uint128::zero(),
            supply: Uint128::zero(),
            decimals,
            supply_denom,
            reserve_denom,
        }
    }
}

pub const CURVE_STATE: Item<CurveState> = Item::new("curve_state");

pub const CURVE_TYPE: Item<CurveType> = Item::new("curve_type");

pub const IS_ACTIVE: Item<bool> = Item::new("is_active");

pub const DISSOLVED_CURVE_STATE: Item<CurveState> = Item::new("dissolved_curve_state");

pub const IS_TEST_MODE: Item<bool> = Item::new("is_test_mode");

pub const IS_SIMULATION_MODE: Item<bool> = Item::new("is_simulation_mode");
