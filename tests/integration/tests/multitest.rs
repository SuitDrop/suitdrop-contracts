use cosmwasm_std::{
    coin, coins, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw_bonding_pool::msg::{CurveType};
use cw_multi_test::{App, ContractWrapper, Executor};
use suitdrop_redeem::{
    msg::{ConfigResponse, RedemptionResponse, RedemptionsResponse},
    state::Redemption,
};

const NORM: u128 = 40_000_000_000_000;

fn create_root_addr() -> Addr {
    Addr::unchecked("root")
}
#[test]
fn test_integration() {
    let mut app = App::default();

    // mint initial supplies of SHIRT and OSMO
    app.init_modules(|router, _api, storage| {
        router
            .bank
            .init_balance(
                storage,
                &create_root_addr(),
                vec![
                    coin(500_000000u128, "ushirt"),
                    coin(500_000_000_000_000_u128, "uosmo"),
                ],
            )
            .unwrap();
    });

    let mock_bonding_code = ContractWrapper::new(
        |_deps: DepsMut,
         _env: Env,
         _info: MessageInfo,
         _msg: cw_bonding_pool::msg::ExecuteMsg|
         -> StdResult<Response> { Ok(Response::new()) },
        |_deps: DepsMut,
         _env: Env,
         _info: MessageInfo,
         _msg: cw_bonding_pool::msg::InstantiateMsg|
         -> StdResult<Response> { Ok(Response::new()) },
        |_deps: Deps, _env: Env, _msg: cw_bonding_pool::msg::QueryMsg| -> StdResult<Binary> {
            Ok(Binary::default())
        },
    );
    let mock_bonding_code_id = app.store_code(Box::new(mock_bonding_code));

    let mock_cw_bonding_addr = app
        .instantiate_contract(
            mock_bonding_code_id,
            create_root_addr(),
            &cw_bonding_pool::msg::InstantiateMsg {
                supply_subdenom: "ushirt".to_string(),
                supply_decimals: 6,
                max_supply: 500_000_000u128.into(),
                reserve_denom: "uosmo".to_string(),
                reserve_decimals: 6,
                curve_type: CurveType::Linear {
                    slope: 1u128.into(),
                    scale: 1u32,
                },
            },
            &[],
            "cw-bonding-pool",
            None,
        )
        .unwrap();

    let cw721_code = ContractWrapper::new(
        cw721_suit::contract::execute,
        cw721_suit::contract::instantiate,
        cw721_suit::contract::query,
    );
    let cw721_code_id = app.store_code(Box::new(cw721_code));

    let suitdrop_redeem_code = ContractWrapper::new(
        suitdrop_redeem::contract::execute,
        suitdrop_redeem::contract::instantiate,
        suitdrop_redeem::contract::query,
    )
    .with_reply(suitdrop_redeem::contract::reply);
    let suitdrop_redeem_code_id = app.store_code(Box::new(suitdrop_redeem_code));

    let cw_bonding_pool_code = ContractWrapper::new(
        cw_bonding_pool::contract::execute,
        cw_bonding_pool::contract::instantiate,
        cw_bonding_pool::contract::query,
    );
    let _cw_bonding_pool_code_id = app.store_code(Box::new(cw_bonding_pool_code));

    // DISABLED FOR NOW. NEED CW-MULTI-TEST TOKEN FACTORY HANDLER OR TO USE TEST TUBE
    // let pool_addr = app
    //     .instantiate_contract(
    //         cw_bonding_pool_code_id,
    //         create_root_addr(),
    // &cw_bonding_pool::msg::InstantiateMsg {
    //     supply_subdenom: "ushirt".to_string(),
    //     supply_decimals: 6,
    //     max_supply: 500_000_000u128.into(),
    //     reserve_denom: "uosmo".to_string(),
    //     reserve_decimals: 6,
    //     curve_type: CurveType::Linear {
    //         slope: 1u128.into(),
    //         scale: 1u32,
    //     },
    // },
    // &[],
    // "cw-bonding-pool",
    // None,
    //     )
    //     .unwrap();

    // let redemption_denom = app
    //     .wrap()
    //     .query_wasm_smart::<GetTotalPoolLiquidityResponse>(
    //         pool_addr,
    //         &cw_bonding_pool::msg::QueryMsg::GetTotalPoolLiquidity {},
    //     )
    //     .unwrap()
    //     .total_pool_liquidity[0]
    //     .denom
    //     .clone();
    let redeem_addr = app
        .instantiate_contract(
            suitdrop_redeem_code_id,
            create_root_addr(),
            &suitdrop_redeem::msg::InstantiateMsg {
                redemption_denom: "ushirt".to_string(),
                nft_code_id: cw721_code_id.into(),
                nft_name: "Shirt NFT".to_string(),
                nft_symbol: "SHIRT".to_string(),
                cost_per_unit: Uint128::from(1_000_000u128),
                // filler
                bonding_contract_addr: mock_cw_bonding_addr.to_string(),
                nft_receipt_token_uri: "https://example.com".to_string(),
            },
            &[],
            "suitdrop-redeem",
            None,
        )
        .unwrap();

    // test queries

    //   QueryMsg::Redemptions { start_after, limit } => {
    //             to_binary(&query_redemptions(deps, env, start_after, limit)?)
    //         }
    //         QueryMsg::Redemption { id, proof } => to_binary(&query_redemption(deps, id, proof)?),
    //         QueryMsg::Config {} => to_binary(&query_config(deps)?),

    let ConfigResponse {
        nft_contract_addr, ..
    } = app
        .wrap()
        .query_wasm_smart(
            redeem_addr.clone(),
            &suitdrop_redeem::msg::QueryMsg::Config {},
        )
        .unwrap();

    let expected_config = ConfigResponse {
        redemption_denom: "ushirt".to_string(),
        cost_per_unit: Uint128::from(1_000_000u128),
        nft_receipt_token_uri: "https://example.com".to_string(),
        // hack to avoid needing to parse the nft contract address from the instantiate response
        nft_contract_addr: nft_contract_addr.clone(),
        bonding_contract_addr: mock_cw_bonding_addr.to_string(),
    };

    let actual_config: ConfigResponse = app
        .wrap()
        .query_wasm_smart(
            redeem_addr.clone(),
            &suitdrop_redeem::msg::QueryMsg::Config {},
        )
        .unwrap();

    assert_eq!(expected_config, actual_config);

    // should fail with insufficient ushirt
    assert!(app
        .execute_contract(
            create_root_addr(),
            redeem_addr.clone(),
            &suitdrop_redeem::msg::ExecuteMsg::Redeem {
                proof: "abcde".to_string(),
            },
            &coins(1u128, "ushirt"),
        )
        .is_err());

    // should fail with excess ushirt
    assert!(app
        .execute_contract(
            create_root_addr(),
            redeem_addr.clone(),
            &suitdrop_redeem::msg::ExecuteMsg::Redeem {
                proof: "abcde".to_string(),
            },
            &coins(1_000_001u128, "ushirt"),
        )
        .is_err());

    // should return empty
    let expected_redemptions: RedemptionsResponse = RedemptionsResponse {
        redemptions: vec![],
    };
    let actual_redemptions: RedemptionsResponse = app
        .wrap()
        .query_wasm_smart(
            redeem_addr.clone(),
            &suitdrop_redeem::msg::QueryMsg::Redemptions {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(expected_redemptions, actual_redemptions);

    // should succeed with sufficient ushirt
    app.execute_contract(
        create_root_addr(),
        redeem_addr.clone(),
        &suitdrop_redeem::msg::ExecuteMsg::Redeem {
            proof: "abcde".to_string(),
        },
        &coins(1_000_000u128, "ushirt"),
    )
    .unwrap();

    // should fail with same proof twice
    assert!(app
        .execute_contract(
            create_root_addr(),
            redeem_addr.clone(),
            &suitdrop_redeem::msg::ExecuteMsg::Redeem {
                proof: "abcde".to_string(),
            },
            &coins(1_000_000u128, "ushirt"),
        )
        .is_err());

    let expected_redemptions: RedemptionsResponse = RedemptionsResponse {
        redemptions: vec![Redemption {
            id: "1".to_string(),
            proof: "abcde".to_string(),
            redeemer: create_root_addr().to_string(),
        }],
    };
    let actual_redemptions: RedemptionsResponse = app
        .wrap()
        .query_wasm_smart(
            redeem_addr.clone(),
            &suitdrop_redeem::msg::QueryMsg::Redemptions {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    assert_eq!(expected_redemptions, actual_redemptions);

    let expected_redemption: RedemptionResponse = RedemptionResponse {
        redemption: Redemption {
            id: "1".to_string(),
            proof: "abcde".to_string(),
            redeemer: create_root_addr().to_string(),
        },
    };
    let actual_redemption: RedemptionResponse = app
        .wrap()
        .query_wasm_smart(
            redeem_addr,
            &suitdrop_redeem::msg::QueryMsg::Redemption {
                id: Some("1".to_string()),
                proof: Some("abcde".to_string()),
            },
        )
        .unwrap();
    assert_eq!(expected_redemption, actual_redemption);

    // should mint nft to redeemer and execute dissolve message on bonding contract
    let expected_nfts = cw721::TokensResponse {
        tokens: vec!["1".to_string()],
    };
    let actual_nfts: cw721::TokensResponse = app
        .wrap()
        .query_wasm_smart(
            nft_contract_addr,
            &cw721_suit::msg::QueryMsg::Tokens {
                owner: create_root_addr().to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(expected_nfts, actual_nfts);
}
