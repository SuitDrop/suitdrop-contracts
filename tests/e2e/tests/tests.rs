use std::{fs, path::Path, str::FromStr};

use cosmwasm_std::{coin, Uint128};
use cw_bonding_pool::msg::{CurveType, ExecuteMsg, GetTotalPoolLiquidityResponse};
use osmosis_test_tube::{Account, Bank, Gamm, Module, OsmosisTestApp, Runner, TokenFactory, Wasm};

pub struct Querier<'a, R: Runner<'a>> {
    runner: &'a R,
}

impl<'a, R: Runner<'a>> Module<'a, R> for Querier<'a, R> {
    fn new(runner: &'a R) -> Self {
        Self { runner }
    }
}

const NORM: u128 = 40_000_000_000_000;

/* 
    Test Targets:

    * Creating Cosmwasm Pool works 
    * More than 500 shirts cannot be minted
    * when all shirts are redeemed, the pool contract balance of both shirt and osmo is zero

*/

#[test]
fn test_tube_integration() {
    let app = OsmosisTestApp::new();

    let acc = app.init_account(&[coin(10 * NORM, "uosmo")]).unwrap();

    let bank = Bank::new(&app);
    let wasm = Wasm::new(&app);
    let fact = TokenFactory::new(&app);
    let gamm = Gamm::new(&app);

    // store codes
    let base_path = Path::new("../target/wasm32-unknown-unknown/release/");
    let cw721_path = base_path.join("cw721_suit.wasm");
    let suitdrop_redeem_path = base_path.join("suitdrop_redeem.wasm");
    let cw_bonding_pool_path = base_path.join("cw_bonding_pool.wasm");

    let cw721_wasm = fs::read(cw721_path).unwrap();
    let cw721_store_resp = wasm.store_code(&cw721_wasm, None, &acc).unwrap();
    let cw721_code = cw721_store_resp.data.code_id;

    let suitdrop_redeem_wasm = fs::read(suitdrop_redeem_path).unwrap();
    let suitdrop_redeem_store_resp = wasm.store_code(&suitdrop_redeem_wasm, None, &acc).unwrap();
    let suitdrop_redeem_code = suitdrop_redeem_store_resp.data.code_id;

    let cw_bonding_pool_wasm = fs::read(cw_bonding_pool_path).unwrap();
    let cw_bonding_pool_store_resp = wasm.store_code(&cw_bonding_pool_wasm, None, &acc).unwrap();
    let cw_bonding_pool_code = cw_bonding_pool_store_resp.data.code_id;

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
