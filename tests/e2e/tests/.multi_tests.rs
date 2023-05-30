use cosmwasm_std::{coin, Uint128};
use cw_bonding_pool::msg::{CurveType, ExecuteMsg, GetTotalPoolLiquidityResponse};
use cw_multi_test::{App, ContractWrapper};

const NORM: u128 = 40_000_000_000_000;

#[test]
fn test_integration() {
    let mut app = App::default();

    let cw721_code = ContractWrapper::new(execute, instantiate, query);
    let cw721_code_id = app.store_code(Box::new(cw721_code));

    let acc = app.init_account(&[coin(10 * NORM, "uosmo")]).unwrap();

    // store codes
    let cw721_store_resp = app.store_code(&cw721_wasm, None, &acc).unwrap();
    let cw721_code = cw721_store_resp.data.code_id;

    let suitdrop_redeem_wasm = fs::read(suitdrop_redeem_path).unwrap();
    let suitdrop_redeem_store_resp = wasm.store_code(&suitdrop_redeem_wasm, None, &acc).unwrap();
    let suitdrop_redeem_code = suitdrop_redeem_store_resp.data.code_id;

    let cw_bonding_pool_wasm = fs::read(cw_bonding_pool_path).unwrap();
    let cw_bonding_pool_store_resp = wasm.store_code(&cw_bonding_pool_wasm, None, &acc).unwrap();
    let cw_bonding_pool_code = cw_bonding_pool_store_resp.data.code_id;

    // let core_addr = wasm
    //     .instantiate(
    //         core_code,
    //         &core::InstantiateMsg {
    //             gov: acc.address(),
    //             fee: core::FeePayload {
    //                 collector: acc.address(),
    //                 mint_fee: Some(Decimal::from_ratio(5u64, 10000u64)),
    //                 burn_fee: Some(Decimal::from_ratio(15u64, 10000u64)),
    //                 streaming_fee: None,
    //             },
    //             index_denom: "uibcx".to_string(),
    //             index_units: vec![
    //                 (uusd.clone(), Decimal::from_str("22.2").unwrap()),
    //                 (ujpy.clone(), Decimal::from_str("20.328").unwrap()),
    //                 (ukrw.clone(), Decimal::from_str("496.225").unwrap()),
    //             ],
    //             reserve_denom: "uosmo".to_string(),
    //         },
    //         Some(&acc.address()),
    //         None,
    //         &[coin(
    //             denom_creation_fee[0].amount.parse().unwrap(),
    //             &denom_creation_fee[0].denom,
    //         )],
    //         &acc,
    //     )
    //     .unwrap()
    //     .data
    //     .address;

    let pool_addr = wasm
        .instantiate(
            cw_bonding_pool_code,
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
            None,
            None,
            &[],
            &acc,
        )
        .unwrap()
        .data
        .address;

    let redemption_denom = wasm
        .query::<_, GetTotalPoolLiquidityResponse>(
            &pool_addr,
            &cw_bonding_pool::msg::QueryMsg::GetTotalPoolLiquidity {},
        )
        .unwrap()
        .total_pool_liquidity[0]
        .denom
        .clone();

    let redeem_addr = wasm
        .instantiate(
            suitdrop_redeem_code,
            &suitdrop_redeem::msg::InstantiateMsg {
                redemption_denom,
                nft_code_id: cw721_code.into(),
                nft_name: "Shirt NFT".to_string(),
                nft_symbol: "SHIRT".to_string(),
            },
            None,
            None,
            &[],
            &acc,
        )
        .unwrap()
        .data
        .address;
}
