


use cosmwasm_std::{
    Coin, Decimal, Deps, StdError, Uint128,
};



use crate::error::ContractError;
use crate::msg::{
    CurveFn,
};
use crate::state::{CurveState, CURVE_STATE, CURVE_TYPE};

// SwapExactAmountIn swaps an exact amount of tokens in for as many tokens out as possible.
/// The amount of tokens out is determined by the current exchange rate and the swap fee.
/// The user specifies a minimum amount of tokens out, and the transaction will revert if that amount of tokens
/// is not received.
pub fn calc_swap_exact_amount_in(
    deps: Deps,
    token_in: Coin,
    token_out_denom: String,
    _swap_fee: Decimal,
) -> Result<(Uint128, CurveState), ContractError> {
    let state = CURVE_STATE.load(deps.storage)?;
    let curve_type = CURVE_TYPE.load(deps.storage)?;
    let curve_fn = curve_type.to_curve_fn();
    // if received reserve token, is buy. if received supply token, is sell.

    if &token_in.denom == &state.reserve_denom {
        if token_out_denom != state.supply_denom {
            return Err(ContractError::Std(StdError::generic_err(
                "invalid token out denom",
            )));
        }
        calc_buy_exact_in(state, curve_fn, token_in.amount)
    } else if &token_in.denom == &state.supply_denom {
        if token_out_denom != state.reserve_denom {
            return Err(ContractError::Std(StdError::generic_err(
                "invalid token out denom",
            )));
        }
        calc_sell_exact_in(state, curve_fn, token_in.amount)
    } else {
        Err(ContractError::Std(StdError::generic_err(
            "invalid token in denom",
        )))
    }
}

pub fn calc_swap_exact_amount_out(
    deps: Deps,
    token_in_denom: String,
    token_out: Coin,
    swap_fee: Decimal,
) -> Result<(Uint128, CurveState), ContractError> {
    if !swap_fee.is_zero() {
        return Err(ContractError::Std(StdError::generic_err(
            "swap fee must be zero",
        )));
    }
    let state = CURVE_STATE.load(deps.storage)?;
    let curve_type = CURVE_TYPE.load(deps.storage)?;
    let curve_fn = curve_type.to_curve_fn();
    // if received reserve token, is buy. if received supply token, is sell.

    if &token_in_denom == &state.reserve_denom {
        if token_out.denom != state.supply_denom {
            return Err(ContractError::Std(StdError::generic_err(
                "invalid token out denom",
            )));
        }

        calc_buy_exact_out(state, curve_fn, token_out.amount)
    } else if &token_in_denom == &state.supply_denom {
        if token_out.denom != state.reserve_denom {
            return Err(ContractError::Std(StdError::generic_err(
                "invalid token out denom",
            )));
        }

        calc_sell_exact_out(state, curve_fn, token_out.amount)
    } else {
        Err(ContractError::Std(StdError::generic_err(
            "invalid token in denom",
        )))
    }
}

pub fn calc_buy_exact_out(
    mut state: CurveState,
    curve_fn: CurveFn,
    mint_amount: Uint128,
) -> Result<(Uint128, CurveState), ContractError> {
    let curve = curve_fn(state.clone().decimals);
    state.supply = state
        .supply
        .checked_add(mint_amount)
        .map_err(StdError::overflow)?;
    let new_reserve = curve.reserve(state.supply);
    let cost = new_reserve
        .checked_sub(state.reserve)
        .map_err(StdError::overflow)?;
    state.reserve = new_reserve;
    let state = state;
    Ok((cost, state))
}

fn calc_sell_exact_out(
    mut state: CurveState,
    curve_fn: CurveFn,
    release_amount: Uint128,
) -> Result<(Uint128, CurveState), ContractError> {
    let curve = curve_fn(state.clone().decimals);
    state.reserve = state
        .reserve
        .checked_sub(release_amount)
        .map_err(StdError::overflow)?;
    let new_supply = curve.supply(state.reserve);
    let burned = state
        .supply
        .checked_sub(new_supply)
        .map_err(StdError::overflow)?;
    state.supply = new_supply;
    Ok((burned, state))
}

pub fn calc_buy_exact_in(
    mut state: CurveState,
    curve_fn: CurveFn,
    payment: Uint128,
) -> Result<(Uint128, CurveState), ContractError> {
    let curve = curve_fn(state.clone().decimals);
    state.reserve += payment;
    let new_supply = curve.supply(state.reserve);
    let minted = new_supply
        .checked_sub(state.supply)
        .map_err(StdError::overflow)?;
    state.supply = new_supply;
    Ok((minted, state))
}

fn calc_sell_exact_in(
    mut state: CurveState,
    curve_fn: CurveFn,
    amount: Uint128,
) -> Result<(Uint128, CurveState), ContractError> {
    let curve = curve_fn(state.clone().decimals);
    state.supply = state
        .supply
        .checked_sub(amount)
        .map_err(StdError::overflow)?;
    let new_reserve = curve.reserve(state.supply);
    let released = state
        .reserve
        .checked_sub(new_reserve)
        .map_err(StdError::overflow)?;
    state.reserve = new_reserve;
    Ok((released, state))
}

#[cfg(test)]
mod tests {
    use crate::{
        curves::{Curve, DecimalPlaces},
        msg::CurveType,
    };
    use cosmwasm_std::Decimal as StdDecimal;
    

    use super::*;

    #[test]
    fn test_curve_calc_mappings() {
        // value of 15 at scale of 1 = 1.5 slope, or 15 / 10
        let curve_type = CurveType::Constant {
            value: 15u128.into(),
            scale: 1,
        };

        // osmo decimals = 6
        // shirt decimals = 0
        let normalize = DecimalPlaces::new(0, 6);
        let state = CurveState {
            decimals: normalize,
            reserve_denom: "osmo".to_string(),
            supply_denom: "shirt".to_string(),
            reserve: 15000000000u128.into(),
            supply: 10000u128.into(),
        };

        let curve = curve_type.to_curve_fn()(state.clone().decimals);

        assert_eq!(
            StdDecimal::percent(150),
            curve.spot_price(Uint128::new(123))
        );

        // FUNCTION: calc_buy_exact_out

        let (cost, out_state) =
            calc_buy_exact_out(state.clone(), curve_type.to_curve_fn(), 1000u128.into()).unwrap();
        assert_eq!(cost.u128(), 1500000000u128, "calc_buy_exact_out");
        assert_eq!(out_state.reserve.u128(), state.reserve.u128() + cost.u128());
        assert_eq!(out_state.supply.u128(), state.supply.u128() + 1000u128);

        // FUNCTION: calc_sell_exact_out

        // amount below precision
        let (burned, out_state) =
            calc_sell_exact_out(state.clone(), curve_type.to_curve_fn(), 1000u128.into()).unwrap();
        assert_eq!(
            burned.u128(),
            1u128,
            "calc_sell_exact_out, below precision 1"
        );
        assert_eq!(out_state.reserve.u128(), state.reserve.u128() - 1000u128);
        assert_eq!(out_state.supply.u128(), state.supply.u128() - burned.u128());

        let (burned, out_state) =
            calc_sell_exact_out(state.clone(), curve_type.to_curve_fn(), 1301u128.into()).unwrap();
        assert_eq!(
            burned.u128(),
            1u128,
            "calc_sell_exact_out, below precision 2"
        );
        assert_eq!(out_state.reserve.u128(), state.reserve.u128() - 1301u128);
        assert_eq!(out_state.supply.u128(), state.supply.u128() - burned.u128());

        // amount above precision
        let (burned, out_state) =
            calc_sell_exact_out(state.clone(), curve_type.to_curve_fn(), 15000000u128.into())
                .unwrap();
        assert_eq!(
            burned.u128(),
            10u128,
            "calc_sell_exact_out, above precision"
        );
        assert_eq!(
            out_state.reserve.u128(),
            state.reserve.u128() - 15000000u128
        );
        assert_eq!(out_state.supply.u128(), state.supply.u128() - burned.u128());

        // calc_buy_exact_in

        let (minted, out_state) =
            calc_buy_exact_in(state.clone(), curve_type.to_curve_fn(), 1000u128.into()).unwrap();
        assert_eq!(minted.u128(), 0u128, "calc_buy_exact_in, below precision");
        assert_eq!(out_state.reserve.u128(), state.reserve.u128() + 1000u128);
        assert_eq!(out_state.supply.u128(), state.supply.u128());

        let (minted, out_state) = calc_buy_exact_in(
            state.clone(),
            curve_type.to_curve_fn(),
            3000000000u128.into(),
        )
        .unwrap();
        assert_eq!(
            minted.u128(),
            2000u128,
            "calc_buy_exact_in, above precision"
        );
        assert_eq!(
            out_state.reserve.u128(),
            state.reserve.u128() + 3000000000u128
        );
        assert_eq!(out_state.supply.u128(), state.supply.u128() + 2000u128);

        // calc_sell_exact_in

        let (released, out_state) =
            calc_sell_exact_in(state.clone(), curve_type.to_curve_fn(), 1000u128.into()).unwrap();
        assert_eq!(released.u128(), 1500000000u128, "calc_sell_exact_in");
        assert_eq!(
            out_state.reserve.u128(),
            state.reserve.u128() - released.u128()
        );
        assert_eq!(out_state.supply.u128(), state.supply.u128() - 1000u128);
    }
}
