use crate::error::ContractError;
use crate::msg::{CurveFn, CurveType};
use crate::state::CurveState;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Decimal, Deps, StdError, Uint128};

// SwapExactAmountIn swaps an exact amount of tokens in for as many tokens out as possible.
/// The amount of tokens out is determined by the current exchange rate and the swap fee.
/// The user specifies a minimum amount of tokens out, and the transaction will revert if that amount of tokens
/// is not received.
pub fn calc_swap_exact_amount_in(
    token_in: Coin,
    token_out_denom: String,
    _swap_fee: Decimal,
    curve_state: CurveState,
    curve_type: CurveType,
) -> Result<(Uint128, CurveState), ContractError> {
    let curve_fn = curve_type.to_curve_fn();
    // if received reserve token, is buy. if received supply token, is sell.

    if &token_in.denom == &curve_state.reserve_denom {
        if token_out_denom != curve_state.supply_denom {
            return Err(ContractError::Std(StdError::generic_err(
                "invalid token out denom",
            )));
        }
        calc_buy_exact_in(curve_state, curve_fn, token_in.amount)
    } else if &token_in.denom == &curve_state.supply_denom {
        if token_out_denom != curve_state.reserve_denom {
            return Err(ContractError::Std(StdError::generic_err(
                "invalid token out denom",
            )));
        }
        calc_sell_exact_in(curve_state, curve_fn, token_in.amount)
    } else {
        Err(ContractError::Std(StdError::generic_err(
            "invalid token in denom",
        )))
    }
}

#[cw_serde]
pub struct CalcSwapExactAmountInRequest {
    pub token_in: Coin,
    pub token_out_denom: String,
    pub swap_fee: Decimal,
    pub curve_state: CurveState,
    pub curve_type: CurveType,
}

impl CalcSwapExactAmountInRequest {
    pub fn execute(self) -> Result<(Uint128, CurveState), ContractError> {
        calc_swap_exact_amount_in(
            self.token_in,
            self.token_out_denom,
            self.swap_fee,
            self.curve_state,
            self.curve_type,
        )
    }
}

pub fn calc_swap_exact_amount_out(
    token_in_denom: String,
    token_out: Coin,
    swap_fee: Decimal,
    curve_state: CurveState,
    curve_type: CurveType,
) -> Result<(Uint128, CurveState), ContractError> {
    if !swap_fee.is_zero() {
        return Err(ContractError::Std(StdError::generic_err(
            "swap fee must be zero",
        )));
    }
    let curve_fn = curve_type.to_curve_fn();
    // if received reserve token, is buy. if received supply token, is sell.

    if &token_in_denom == &curve_state.reserve_denom {
        if token_out.denom != curve_state.supply_denom {
            return Err(ContractError::Std(StdError::generic_err(
                "invalid token out denom",
            )));
        }

        calc_buy_exact_out(curve_state, curve_fn, token_out.amount)
    } else if &token_in_denom == &curve_state.supply_denom {
        if token_out.denom != curve_state.reserve_denom {
            return Err(ContractError::Std(StdError::generic_err(
                "invalid token out denom",
            )));
        }

        calc_sell_exact_out(curve_state, curve_fn, token_out.amount)
    } else {
        Err(ContractError::Std(StdError::generic_err(
            "invalid token in denom",
        )))
    }
}

#[cw_serde]
pub struct CalcSwapExactAmountOutRequest {
    pub token_in_denom: String,
    pub token_out: Coin,
    pub swap_fee: Decimal,
    pub curve_state: CurveState,
    pub curve_type: CurveType,
}

impl CalcSwapExactAmountOutRequest {
    pub fn execute(self) -> Result<(Uint128, CurveState), ContractError> {
        calc_swap_exact_amount_out(
            self.token_in_denom,
            self.token_out,
            self.swap_fee,
            self.curve_state,
            self.curve_type,
        )
    }
}

pub fn calc_spot_price(
    quote_asset_denom: String,
    base_asset_denom: String,
    curve_state: CurveState,
    curve_type: CurveType,
) -> Result<Decimal, ContractError> {
    let curve_fn = curve_type.to_curve_fn();
    let curve = curve_fn(curve_state.clone().decimals);
    let mut spot_price = curve.spot_price(curve_state.supply);

    // quote denom must not equal base denom.
    if quote_asset_denom == base_asset_denom {
        return Err(ContractError::Std(StdError::generic_err(
            "quote denom must not equal base denom",
        )));
    }

    // one of the assets must be the reserve asset.
    if quote_asset_denom != curve_state.reserve_denom
        && base_asset_denom != curve_state.reserve_denom
    {
        return Err(ContractError::Std(StdError::generic_err(
            "one of the assets must be the reserve asset",
        )));
    }

    // one of the assets must be the supply asset
    if quote_asset_denom != curve_state.supply_denom && base_asset_denom != curve_state.supply_denom
    {
        return Err(ContractError::Std(StdError::generic_err(
            "one of the assets must be the supply asset",
        )));
    }

    if quote_asset_denom != curve_state.reserve_denom {
        spot_price = Decimal::one().checked_div(spot_price)?;
    }

    Ok(spot_price)
}

#[cw_serde]
pub struct CalcSpotPriceRequest {
    pub quote_asset_denom: String,
    pub base_asset_denom: String,
    pub curve_state: CurveState,
    pub curve_type: CurveType,
}

impl CalcSpotPriceRequest {
    pub fn execute(self) -> Result<Decimal, ContractError> {
        calc_spot_price(
            self.quote_asset_denom,
            self.base_asset_denom,
            self.curve_state,
            self.curve_type,
        )
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

/// CLIENT-SIDE FOCUSED CALCULATIONS

#[cw_serde]
pub struct Quote {
    pub amount: Uint128,
    pub before_spot_price_in_over_out: Decimal,
    pub before_spot_price_out_over_in: Decimal,
    pub after_spot_price_in_over_out: Decimal,
    pub after_spot_price_out_over_in: Decimal,
    pub effective_price_in_over_out: Decimal,
    pub effective_price_out_over_in: Decimal,
    pub price_impact_token_out: Decimal,
    pub num_ticks_crossed: Option<u128>,
}

pub fn get_token_in_by_token_out(
    token_out: Coin,
    token_in_denom: String,
    swap_fee: Decimal,
    curve_state: CurveState,
    curve_type: CurveType,
) -> Result<Quote, ContractError> {
    let before_spot_price_in_over_out = calc_spot_price(
        token_in_denom.clone(),
        token_out.denom.clone(),
        curve_state.clone(),
        curve_type.clone(),
    )?;
    let before_spot_price_out_over_in =
        Decimal::one().checked_div(before_spot_price_in_over_out)?;

    let (amount_in, out_state) = calc_swap_exact_amount_out(
        token_in_denom.clone(),
        token_out.clone(),
        swap_fee,
        curve_state,
        curve_type.clone(),
    )?;
    let amount_out = token_out.amount;

    let after_spot_price_in_over_out = calc_spot_price(
        token_in_denom.clone(),
        token_out.denom.clone(),
        out_state.clone(),
        curve_type.clone(),
    )?;
    let after_spot_price_out_over_in = Decimal::one().checked_div(after_spot_price_in_over_out)?;

    let effective_price_in_over_out = Decimal::from_ratio(amount_in, amount_out);
    let effective_price_out_over_in = Decimal::one().checked_div(effective_price_in_over_out)?;
    let price_impact_token_out = effective_price_in_over_out
        .checked_div(before_spot_price_in_over_out)?
        .checked_sub(Decimal::one())?;
    let num_ticks_crossed = None;

    Ok(Quote {
        amount: amount_in,
        before_spot_price_in_over_out,
        before_spot_price_out_over_in,
        after_spot_price_in_over_out,
        after_spot_price_out_over_in,
        effective_price_in_over_out,
        effective_price_out_over_in,
        price_impact_token_out,
        num_ticks_crossed,
    })
}

#[cw_serde]
pub struct GetTokenInByTokenOutRequest {
    pub token_out: Coin,
    pub token_in_denom: String,
    pub swap_fee: Decimal,
    pub curve_state: CurveState,
    pub curve_type: CurveType,
}

impl GetTokenInByTokenOutRequest {
    pub fn execute(self) -> Result<Quote, ContractError> {
        get_token_in_by_token_out(
            self.token_out,
            self.token_in_denom,
            self.swap_fee,
            self.curve_state,
            self.curve_type,
        )
    }
}

pub fn get_token_out_by_token_in(
    token_in: Coin,
    token_out_denom: String,
    swap_fee: Decimal,
    curve_state: CurveState,
    curve_type: CurveType,
) -> Result<Quote, ContractError> {
    let before_spot_price_in_over_out = calc_spot_price(
        token_in.denom.clone(),
        token_out_denom.clone(),
        curve_state.clone(),
        curve_type.clone(),
    )?;
    let before_spot_price_out_over_in =
        Decimal::one().checked_div(before_spot_price_in_over_out)?;

    let (amount_out, out_state) = calc_swap_exact_amount_in(
        token_in.clone(),
        token_out_denom.clone(),
        swap_fee,
        curve_state,
        curve_type.clone(),
    )?;
    let amount_in = token_in.amount;

    let after_spot_price_in_over_out = calc_spot_price(
        token_in.denom.clone(),
        token_out_denom.clone(),
        out_state.clone(),
        curve_type,
    )?;
    let after_spot_price_out_over_in = Decimal::one().checked_div(after_spot_price_in_over_out)?;

    let effective_price_in_over_out = Decimal::from_ratio(amount_in, amount_out);
    let effective_price_out_over_in = Decimal::one().checked_div(effective_price_in_over_out)?;
    let price_impact_token_out = effective_price_in_over_out
        .checked_div(before_spot_price_in_over_out)?
        .checked_sub(Decimal::one())?;
    let num_ticks_crossed = None;

    Ok(Quote {
        amount: amount_out,
        before_spot_price_in_over_out,
        before_spot_price_out_over_in,
        after_spot_price_in_over_out,
        after_spot_price_out_over_in,
        effective_price_in_over_out,
        effective_price_out_over_in,
        price_impact_token_out,
        num_ticks_crossed,
    })
}

#[cw_serde]
pub struct GetTokenOutByTokenInRequest {
    pub token_in: Coin,
    pub token_out_denom: String,
    pub swap_fee: Decimal,
    pub curve_state: CurveState,
    pub curve_type: CurveType,
}

impl GetTokenOutByTokenInRequest {
    pub fn execute(self) -> Result<Quote, ContractError> {
        get_token_out_by_token_in(
            self.token_in,
            self.token_out_denom,
            self.swap_fee,
            self.curve_state,
            self.curve_type,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        curves::{Curve, DecimalPlaces},
        msg::CurveType,
    };
    use cosmwasm_std::{coin, Decimal as StdDecimal};

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

    #[test]
    fn test_get_token_in_by_token_out() {
        let curve_type = CurveType::Linear {
            slope: 20u128.into(),
            scale: 1,
        };

        let normalize = DecimalPlaces::new(0, 0);
        let curve_fn = curve_type.to_curve_fn();
        let supply = 10u128;
        let reserve = curve_fn(normalize.clone()).reserve(supply.into());
        let supply = curve_fn(normalize.clone()).supply(reserve);
        assert_eq!(
            reserve,
            curve_fn(normalize.clone()).reserve(supply),
            "reserve and calculated reserve should be equal. if not, test configuration for supply is prone to precision errors."
        );
        let curve_state = CurveState {
            decimals: normalize.clone(),
            reserve_denom: "osmo".to_string(),
            supply_denom: "shirt".to_string(),
            reserve,
            supply,
        };

        let amount_out = 2u128;
        let quote = get_token_in_by_token_out(
            coin(amount_out.clone(), "shirt"),
            "osmo".to_string(),
            Decimal::zero(),
            curve_state,
            curve_type,
        )
        .unwrap();

        assert!(
            quote.amount.u128() > amount_out,
            "quote amount in should be greater than the amount out"
        );
        assert!(
            quote.before_spot_price_in_over_out < quote.after_spot_price_in_over_out,
            "before_spot_price_in_over_out should be less than after_spot_price_in_over_out"
        );
        assert!(
            quote.before_spot_price_out_over_in > quote.after_spot_price_out_over_in,
            "before_spot_price_out_over_in should be greater than after_spot_price_out_over_in"
        );
        assert!(
            quote.effective_price_in_over_out > quote.effective_price_out_over_in,
            "effective_price_in_over_out should be greater than effective_price_out_over_in"
        );
        assert!(
            quote.before_spot_price_in_over_out > quote.before_spot_price_out_over_in,
            "before_spot_price_in_over_out should be greater than before_spot_price_out_over_in"
        );
        assert!(
            quote.after_spot_price_in_over_out > quote.after_spot_price_out_over_in,
            "after_spot_price_in_over_out should be greater than after_spot_price_out_over_in"
        );
        assert!(
            quote.price_impact_token_out > Decimal::zero(),
            "price_impact_token_out should be greater than zero"
        );
        assert_eq!(
            quote.num_ticks_crossed, None,
            "num_ticks_crossed should be None"
        );

        // We already have tests for the math. We just need to make sure the numbers were put in correctly.
        // amount = 4
        // before_spot_price_in_over_out = 2
        // before_spot_price_out_over_in = 0.5
        // after_spot_price_in_over_out = 4
        // after_spot_price_out_over_in = 0.25
        // effective_price_in_over_out = 0.4
        // effective_price_out_over_in = 2.5
        // price_impact_token_out = 0.2
        // num_ticks_crossed = None

        // organize the above properties by whether they should be < > or == each other
    }

    #[test]
    fn test_get_token_out_by_token_in() {
        let curve_type = CurveType::Linear {
            slope: 20u128.into(),
            scale: 1,
        };

        // osmo decimals = 6
        // shirt decimals = 0
        let normalize = DecimalPlaces::new(0, 0);
        let curve_fn = curve_type.to_curve_fn();
        let supply = 10u128;
        let reserve = curve_fn(normalize.clone()).reserve(supply.into());
        let supply = curve_fn(normalize.clone()).supply(reserve);
        assert_eq!(
            reserve,
            curve_fn(normalize.clone()).reserve(supply),
            "reserve and calculated reserve should be equal. if not, test configuration for supply is prone to precision errors."
        );
        let curve_state = CurveState {
            decimals: normalize.clone(),
            reserve_denom: "osmo".to_string(),
            supply_denom: "shirt".to_string(),
            reserve,
            supply,
        };

        let amount_in = 2u128;
        let quote = get_token_out_by_token_in(
            coin(amount_in.clone(), "shirt"),
            "osmo".to_string(),
            Decimal::zero(),
            curve_state,
            curve_type,
        )
        .unwrap();

        assert!(
            quote.amount.u128() > amount_in,
            "quote amount in should be greater than the amount in"
        );
        assert!(
            quote.before_spot_price_in_over_out < quote.after_spot_price_in_over_out,
            "before_spot_price_in_over_out should be less than after_spot_price_in_over_out"
        );
        assert!(
            quote.before_spot_price_out_over_in > quote.after_spot_price_out_over_in,
            "before_spot_price_out_over_in should be greater than after_spot_price_out_over_in"
        );
        assert!(
            quote.effective_price_in_over_out < quote.effective_price_out_over_in,
            "effective_price_in_over_out should be less than effective_price_out_over_in"
        );
        assert!(
            quote.before_spot_price_in_over_out < quote.before_spot_price_out_over_in,
            "before_spot_price_in_over_out should be less than before_spot_price_out_over_in"
        );
        assert!(
            quote.after_spot_price_in_over_out < quote.after_spot_price_out_over_in,
            "after_spot_price_in_over_out should be less than after_spot_price_out_over_in"
        );
        assert!(
            quote.price_impact_token_out > Decimal::zero(),
            "price_impact_token_out should be greater than zero"
        );
        assert_eq!(
            quote.num_ticks_crossed, None,
            "num_ticks_crossed should be None"
        );
    }
}
