#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Order, Reply,
    Response, StdError, StdResult, SubMsg, Uint64, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::{must_pay, one_coin, parse_reply_instantiate_data};

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, RedemptionResponse,
    RedemptionsResponse,
};
use crate::state::{
    Redemption, RedemptionState, BONDING_CONTRACT, COST_PER_UNIT, NFT_CONTRACT,
    NFT_RECEIPT_TOKEN_URI, REDEMPTION_DENOM, REDEMPTION_INCREMENT,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:suitdrop-redeem";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_NFT_REPLY_ID: u64 = 1;

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let checked_bonding_contract_addr = deps.api.addr_validate(&msg.bonding_contract_addr)?;
    REDEMPTION_DENOM.save(deps.storage, &msg.redemption_denom)?;
    COST_PER_UNIT.save(deps.storage, &msg.cost_per_unit)?;
    BONDING_CONTRACT.save(deps.storage, &checked_bonding_contract_addr)?;
    NFT_RECEIPT_TOKEN_URI.save(deps.storage, &msg.nft_receipt_token_uri)?;
    REDEMPTION_INCREMENT.save(deps.storage, &Uint64::zero())?;

    let wasm_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.nft_code_id.u64(),
        msg: to_binary(&cw721_suit::msg::InstantiateMsg {
            name: msg.nft_name,
            symbol: msg.nft_symbol.clone(),
            minter: env.contract.address.to_string(),
        })?,
        funds: vec![],
        label: format!("SUITDROP-CW721-{}", msg.nft_symbol),
    });

    let wasm_msg = SubMsg::reply_on_success(wasm_msg, INSTANTIATE_NFT_REPLY_ID);

    // With `Response` type, it is possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    Ok(Response::new()
        .add_submessage(wasm_msg)
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

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Redeem { proof } => execute_redeem(deps, env, info, proof),
    }
}

pub fn execute_redeem(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proof: String,
) -> Result<Response, ContractError> {
    one_coin(&info)?;
    let denom = REDEMPTION_DENOM.load(deps.storage)?;
    let bonding_contract = BONDING_CONTRACT.load(deps.storage)?;

    let amount = must_pay(&info, &denom)?;
    if amount != COST_PER_UNIT.load(deps.storage)? {
        return Err(ContractError::InvalidRedemptionAmount { denom });
    }
    let redemption_state = RedemptionState::default();
    let redemption_incr = REDEMPTION_INCREMENT.update(
        deps.storage,
        |redemption_increment| -> Result<Uint64, ContractError> {
            Ok(redemption_increment.checked_add(Uint64::one())?)
        },
    )?;

    let redemption = Redemption {
        id: redemption_incr.to_string(),
        proof,
        redeemer: info.sender.to_string(),
    };

    redemption_state
        .redemptions
        .save(deps.storage, redemption_incr.to_string(), &redemption)?;

    let dissolve_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: bonding_contract.to_string(),
        msg: to_binary(&cw_bonding_pool::msg::ExecuteMsg::Dissolve {})?,
        funds: coins(amount.u128(), &denom),
    });

    let mint_nft_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: NFT_CONTRACT.load(deps.storage)?.to_string(),
        msg: to_binary(&cw721_suit::msg::ExecuteMsg::Mint {
            owner: info.sender.to_string(),
            token_id: redemption_incr.to_string(),
            token_uri: Some(NFT_RECEIPT_TOKEN_URI.load(deps.storage)?),
            extension: None,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(dissolve_msg)
        .add_message(mint_nft_msg)
        .add_attribute("method", "redeem")
        .add_attribute("id", redemption_incr.to_string()))
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Redemptions { start_after, limit } => {
            to_binary(&query_redemptions(deps, env, start_after, limit)?)
        }
        QueryMsg::Redemption { id, proof } => to_binary(&query_redemption(deps, id, proof)?),
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    Ok(ConfigResponse {
        redemption_denom: REDEMPTION_DENOM.load(deps.storage)?,
        cost_per_unit: COST_PER_UNIT.load(deps.storage)?,
        nft_receipt_token_uri: NFT_RECEIPT_TOKEN_URI.load(deps.storage)?,
        nft_contract_addr: NFT_CONTRACT.load(deps.storage)?.to_string(),
        bonding_contract_addr: BONDING_CONTRACT.load(deps.storage)?.to_string(),
    })
}

pub fn query_redemptions(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<RedemptionsResponse> {
    let limit = limit.unwrap_or(30) as usize;
    let redemptions: Vec<Redemption> = RedemptionState::default()
        .redemptions
        .range(
            deps.storage,
            Some(Bound::exclusive(start_after.unwrap_or_default())),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;
    Ok(RedemptionsResponse { redemptions })
}

pub fn query_redemption(
    deps: Deps,
    id: Option<String>,
    proof: Option<String>,
) -> StdResult<RedemptionResponse> {
    if let Some(id) = id {
        let redemption = RedemptionState::default()
            .redemptions
            .load(deps.storage, id)?;
        Ok(RedemptionResponse { redemption })
    } else if let Some(proof) = proof {
        let (_, redemption) = RedemptionState::default()
            .redemptions
            .idx
            .proof
            .item(deps.storage, proof)?
            .ok_or(StdError::not_found("redemption"))?; // TODO: better error message
        Ok(RedemptionResponse { redemption })
    } else {
        Err(StdError::generic_err("Invalid query"))
    }
}

/// Handling submessage reply.
/// For more info on submessage and reply, see https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#submessages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INSTANTIATE_NFT_REPLY_ID => {
            let parsed = parse_reply_instantiate_data(msg)?;
            let nft_addr_verified = deps.api.addr_validate(parsed.contract_address.as_str())?;
            let cw721_base::MinterResponse { minter } = deps.querier.query_wasm_smart(
                nft_addr_verified.clone(),
                &cw721_suit::msg::QueryMsg::Minter {},
            )?;

            if let Some(minter) = minter {
                if minter != env.contract.address {
                    return Err(ContractError::Unauthorized {});
                }
            } else {
                return Err(ContractError::Unauthorized {});
            }
            NFT_CONTRACT.save(deps.storage, &nft_addr_verified)?;
            Ok(Response::new()
                .add_attribute("method", "instantiate_nft")
                .add_attribute("nft_contract", nft_addr_verified.to_string()))
        }
        _ => Err(ContractError::InvalidReplyId {}),
    }
}
