#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, coins, ensure, to_binary, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps,
    DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_utils::{must_pay, one_coin};

use crate::calc::{calc_buy_exact_out, calc_swap_exact_amount_in, calc_swap_exact_amount_out};
use crate::curves::DecimalPlaces;
use crate::error::ContractError;
use crate::helpers::mint_or_send;
use crate::msg::{
    BondingPoolState, CalcInAmtGivenOutResponse, CalcOutAmtGivenInResponse, CurveType, ExecuteMsg,
    GetSwapFeeResponse, GetTotalPoolLiquidityResponse, InstantiateMsg, IsActiveResponse,
    MigrateMsg, QueryMsg, SimulationMsg, SpotPriceResponse, SudoMsg, SwapExactAmountInResponseData,
    SwapExactAmountOutResponseData,
};
use crate::state::{
    CurveState, CURVE_STATE, CURVE_TYPE, DISSOLVED_CURVE_STATE, IS_ACTIVE, IS_SIMULATION_MODE,
    IS_TEST_MODE,
};
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenom;


// version info for migration info
pub const CONTRACT_NAME: &str = "crates.io:cw-bonding-pool";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let supply_denom = format!(
        "factory/{}/{}",
        env.contract.address,
        msg.supply_subdenom
    );
    let places = DecimalPlaces::new(msg.supply_decimals, msg.reserve_decimals);
    let supply = CurveState::new(msg.reserve_denom, supply_denom, places);

    DISSOLVED_CURVE_STATE.save(deps.storage, &supply)?;
    CURVE_STATE.save(deps.storage, &supply)?;
    CURVE_TYPE.save(deps.storage, &msg.curve_type)?;
    IS_ACTIVE.save(deps.storage, &true)?;
    IS_TEST_MODE.save(deps.storage, &msg.test_mode.unwrap_or(false))?;
    IS_SIMULATION_MODE.save(deps.storage, &msg.simulation_mode.unwrap_or(false))?;

    if msg.simulation_mode.unwrap_or(false) {
        // return without executing messages if in simulation mode
        return Ok(Response::new()
            .add_attribute("method", "instantiate")
            .add_attribute("owner", info.sender));
    }

    let msg_create_denom: CosmosMsg = MsgCreateDenom {
        sender: env.contract.address.to_string(),
        subdenom: msg.supply_subdenom,
    }
    .into();

    // With `Response` type, it is possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    Ok(Response::new()
        .add_message(msg_create_denom)
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Handling contract migration
/// To make a contract migratable, you need
/// - this entry_point implemented
/// - only contract admin can migrate, so admin has to be set at contract initiation time
/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        // Find matched incoming message variant and execute them with your custom logic.
        //
        // With `Response` type, it is possible to dispatch message to invoke external logic.
        // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Dissolve { .. } => execute_dissolve(deps, env, info),
        ExecuteMsg::Sudo(sudo_msg) => {
            ensure!(
                IS_TEST_MODE.may_load(deps.storage)?.unwrap_or(false),
                ContractError::Unauthorized {}
            );
            sudo(deps, env, sudo_msg)
        }
        ExecuteMsg::Simulate(sim_msg) => {
            ensure!(
                IS_SIMULATION_MODE.may_load(deps.storage)?.unwrap_or(false),
                ContractError::Unauthorized {}
            );
            execute_simulate(deps, env, sim_msg)
        }
    }
}

pub fn execute_simulate(
    deps: DepsMut,
    _env: Env,
    msg: SimulationMsg,
) -> Result<Response, ContractError> {
    match msg {
        SimulationMsg::SetState { state } => {
            CURVE_STATE.save(deps.storage, &state.curve_state)?;
            DISSOLVED_CURVE_STATE.save(deps.storage, &state.dissolved_curve_state)?;
            CURVE_TYPE.save(deps.storage, &state.curve_type)?;
            IS_ACTIVE.save(deps.storage, &state.is_active)?;
            Ok(Response::new().add_attribute("method", "simulate"))
        }
    }
}

pub fn execute_dissolve(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let curve_state = CURVE_STATE.load(deps.storage)?;
    let curve = CURVE_TYPE.load(deps.storage)?;
    let curve_fn = curve.to_curve_fn();
    one_coin(&info)?;
    let paid = must_pay(&info, &curve_state.supply_denom)?;
    let dissolved_curve_state = DISSOLVED_CURVE_STATE.load(deps.storage)?;

    let (dissolved_reserve_cost, next_dissolved_curve_state) =
        calc_buy_exact_out(dissolved_curve_state, curve_fn, paid)?;

    DISSOLVED_CURVE_STATE.save(deps.storage, &next_dissolved_curve_state)?;

    let mut messages: Vec<cosmwasm_std::CosmosMsg> = vec![];

    if dissolved_reserve_cost > Uint128::zero() {
        messages.push(
            BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: coins(
                    dissolved_reserve_cost.u128(),
                    curve_state.reserve_denom.clone(),
                ),
            }
            .into(),
        );
    }

    Ok(Response::new()
        .add_attribute("method", "dissolve")
        .add_attribute("sender", info.sender)
        .add_attribute("reserve_denom", curve_state.reserve_denom)
        .add_attribute("supply_denom", curve_state.supply_denom)
        .add_attribute("dissolved", paid)
        .add_attribute("distributed", dissolved_reserve_cost)
        .add_messages(messages))
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    // if the message is not SetActive, ensure that the contract is active
    if !matches!(msg, SudoMsg::SetActive { .. }) && !IS_ACTIVE.load(deps.storage)? {
        return Err(ContractError::Std(StdError::generic_err(
            "Contract is not active",
        )));
    }
    match msg {
        SudoMsg::SwapExactAmountIn {
            sender,
            token_in,
            token_out_denom,
            token_out_min_amount,
            swap_fee,
        } => execute_swap_exact_amount_in(
            deps,
            env,
            sender,
            token_in,
            token_out_denom,
            token_out_min_amount,
            swap_fee,
        ),
        SudoMsg::SwapExactAmountOut {
            sender,
            token_in_denom,
            token_in_max_amount,
            token_out,
            swap_fee,
        } => execute_swap_exact_amount_out(
            deps,
            env,
            sender,
            token_in_denom,
            token_in_max_amount,
            token_out,
            swap_fee,
        ),
        SudoMsg::SetActive { is_active } => execute_set_active(deps, env, is_active),
        // Find matched incoming message variant and execute them with your custom logic.
        //
        // With `Response` type, it is possible to dispatch message to invoke external logic.
        // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    }
}

pub fn execute_set_active(
    deps: DepsMut,
    _env: Env,
    is_active: bool,
) -> Result<Response, ContractError> {
    IS_ACTIVE.save(deps.storage, &is_active)?;

    Ok(Response::new()
        .add_attribute("method", "set_active")
        .add_attribute("is_active", is_active.to_string()))
}

/// SwapExactAmountIn swaps an exact amount of tokens in for as many tokens out as possible.
/// The amount of tokens out is determined by the current exchange rate and the swap fee.
/// The user specifies a minimum amount of tokens out, and the transaction will revert if that amount of tokens
/// is not received.
pub fn execute_swap_exact_amount_in(
    deps: DepsMut,
    env: Env,
    sender: String,
    token_in: Coin,
    token_out_denom: String,
    token_out_min_amount: Uint128,
    swap_fee: Decimal,
) -> Result<Response, ContractError> {
    let (token_out_amount, curve) = calc_swap_exact_amount_in(
        deps.as_ref(),
        token_in,
        token_out_denom.clone(),
        swap_fee,
        CURVE_STATE.load(deps.storage)?,
        CURVE_TYPE.load(deps.storage)?,
    )?;

    if token_out_amount < token_out_min_amount {
        return Err(ContractError::Std(StdError::generic_err(
            "insufficient output amount",
        )));
    }

    CURVE_STATE.save(deps.storage, &curve)?;

    let send_msg = mint_or_send(
        curve.supply_denom,
        token_out_denom,
        token_out_amount,
        sender,
        env.contract.address.to_string(),
    );

    let swap_result = SwapExactAmountInResponseData { token_out_amount };

    Ok(Response::new()
        .add_attribute("method", "swap_exact_amount_in")
        .add_message(send_msg)
        .set_data(to_binary(&swap_result)?))
}

/// SwapExactAmountOut swaps as many tokens in as possible for an exact amount of tokens out.
/// The amount of tokens in is determined by the current exchange rate and the swap fee.
/// The user specifies a maximum amount of tokens in, and the transaction will revert if that amount of tokens
/// is exceeded.
pub fn execute_swap_exact_amount_out(
    deps: DepsMut,
    _env: Env,
    sender: String,
    token_in_denom: String,
    token_in_max_amount: Uint128,
    token_out: Coin,
    swap_fee: Decimal,
) -> Result<Response, ContractError> {
    let (token_in_amount, state) = calc_swap_exact_amount_out(
        deps.as_ref(),
        token_in_denom,
        token_out.clone(),
        swap_fee,
        CURVE_STATE.load(deps.storage)?,
        CURVE_TYPE.load(deps.storage)?,
    )?;

    CURVE_STATE.save(deps.storage, &state)?;

    if token_in_amount > token_in_max_amount {
        return Err(ContractError::Std(StdError::generic_err(
            "token in amount exceeds max amount",
        )));
    };
    let send_token_out_to_sender_msg = BankMsg::Send {
        to_address: sender,
        amount: vec![token_out],
    };

    let swap_result = SwapExactAmountOutResponseData { token_in_amount };

    Ok(Response::new()
        .add_attribute("method", "swap_exact_amount_out")
        .add_message(send_token_out_to_sender_msg)
        .set_data(to_binary(&swap_result)?))
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let curve_state = || CURVE_STATE.load(deps.storage);
    let curve_type = || CURVE_TYPE.load(deps.storage);
    let is_active = || IS_ACTIVE.load(deps.storage);

    query_pool_msg_with_state(deps, env, msg, &curve_state, &curve_type, &is_active)
}

pub fn query_pool_msg_with_state(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
    curve_state: &dyn Fn() -> StdResult<CurveState>,
    curve_type: &dyn Fn() -> StdResult<CurveType>,
    is_active: &dyn Fn() -> StdResult<bool>,
) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetSwapFee {} => to_binary(&query_get_swap_fee(deps, env)?),
        QueryMsg::IsActive {} => to_binary(&query_is_active(deps, env, is_active()?)?),
        QueryMsg::GetTotalPoolLiquidity {} => {
            to_binary(&query_get_total_pool_liquidity(deps, env, curve_state()?)?)
        }
        QueryMsg::SpotPrice {
            quote_asset_denom,
            base_asset_denom,
        } => to_binary(&query_spot_price(
            deps,
            env,
            quote_asset_denom,
            base_asset_denom,
            curve_state()?,
            curve_type()?,
        )?),
        QueryMsg::CalcOutAmtGivenIn {
            token_in,
            token_out_denom,
            swap_fee,
        } => to_binary(&query_calc_out_amt_given_in(
            deps,
            env,
            token_in,
            token_out_denom,
            swap_fee,
            curve_state()?,
            curve_type()?,
        )?),
        QueryMsg::CalcInAmtGivenOut {
            token_out,
            token_in_denom,
            swap_fee,
        } => to_binary(&query_calc_in_amt_given_out(
            deps,
            env,
            token_out,
            token_in_denom,
            swap_fee,
            curve_state()?,
            curve_type()?,
        )?),
        QueryMsg::BondingPoolState {} => to_binary(&BondingPoolState {
            curve_state: curve_state()?,
            dissolved_curve_state: DISSOLVED_CURVE_STATE.load(deps.storage)?,
            curve_type: curve_type()?,
            is_active: is_active()?,
        }),
        // Find matched incoming message variant and query them your custom logic
        // and then construct your query response with the type usually defined
        // `msg.rs` alongside with the query message itself.
        //
        // use `cosmwasm_std::to_binary` to serialize query response to json binary.
    }
    .map_err(ContractError::Std)
}

pub fn query_get_swap_fee(_deps: Deps, _env: Env) -> StdResult<GetSwapFeeResponse> {
    Ok(GetSwapFeeResponse {
        swap_fee: Decimal::zero(),
    })
}

pub fn query_is_active(deps: Deps, _env: Env, _is_active: bool) -> StdResult<IsActiveResponse> {
    Ok(IsActiveResponse {
        is_active: IS_ACTIVE.load(deps.storage)?,
    })
}

pub fn query_get_total_pool_liquidity(
    _deps: Deps,
    _env: Env,
    curve_state: CurveState,
) -> Result<GetTotalPoolLiquidityResponse, ContractError> {
    Ok(GetTotalPoolLiquidityResponse {
        total_pool_liquidity: vec![
            coin(curve_state.supply.u128(), curve_state.supply_denom),
            coin(curve_state.reserve.u128(), curve_state.reserve_denom),
        ],
    })
}

pub fn query_spot_price(
    _deps: Deps,
    _env: Env,
    quote_asset_denom: String,
    base_asset_denom: String,
    curve_state: CurveState,
    curve_type: CurveType,
) -> Result<SpotPriceResponse, ContractError> {
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

    Ok(SpotPriceResponse { spot_price })
}

pub fn query_calc_out_amt_given_in(
    deps: Deps,
    _env: Env,
    token_in: Coin,
    token_out_denom: String,
    swap_fee: Decimal,
    curve_state: CurveState,
    curve_type: CurveType,
) -> Result<CalcOutAmtGivenInResponse, ContractError> {
    let (token_out_amount, _state) = calc_swap_exact_amount_in(
        deps,
        token_in,
        token_out_denom.clone(),
        swap_fee,
        curve_state,
        curve_type,
    )?;
    Ok(CalcOutAmtGivenInResponse {
        token_out: coin(token_out_amount.u128(), token_out_denom),
    })
}

pub fn query_calc_in_amt_given_out(
    deps: Deps,
    _env: Env,
    token_out: Coin,
    token_in_denom: String,
    swap_fee: Decimal,
    curve_state: CurveState,
    curve_type: CurveType,
) -> Result<CalcInAmtGivenOutResponse, ContractError> {
    let (token_in_amount, _state) = calc_swap_exact_amount_out(
        deps,
        token_in_denom.clone(),
        token_out,
        swap_fee,
        curve_state,
        curve_type,
    )?;
    Ok(CalcInAmtGivenOutResponse {
        token_in: coin(token_in_amount.u128(), token_in_denom),
    })
}

/// Handling submessage reply.
/// For more info on submessage and reply, see https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#submessages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    // With `Response` type, it is still possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages

    Ok(Response::new())
}
