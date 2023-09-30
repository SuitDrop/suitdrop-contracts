use cw_bonding_pool::calc::{
    CalcSpotPriceRequest, CalcSwapExactAmountInRequest, CalcSwapExactAmountOutRequest,
    GetTokenInByTokenOutRequest, GetTokenOutByTokenInRequest,
};
use wasm_bindgen::prelude::*;
mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global	// allocator.
// allocator.	#[cfg(feature = "wee_alloc")]

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn calc_swap_exact_amount_in(val: JsValue) -> Result<JsValue, JsValue> {
    let request: CalcSwapExactAmountInRequest = serde_wasm_bindgen::from_value(val)?;
    let response = request
        .execute()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    /* …do something with `example`… */
    Ok(serde_wasm_bindgen::to_value(&response)?)
}

#[wasm_bindgen]
pub fn calc_swap_exact_amount_out(val: JsValue) -> Result<JsValue, JsValue> {
    let request: CalcSwapExactAmountOutRequest = serde_wasm_bindgen::from_value(val)?;
    let response = request
        .execute()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    /* …do something with `example`… */
    Ok(serde_wasm_bindgen::to_value(&response)?)
}

#[wasm_bindgen]
pub fn calc_spot_price(val: JsValue) -> Result<JsValue, JsValue> {
    let request: CalcSpotPriceRequest = serde_wasm_bindgen::from_value(val)?;
    let response = request
        .execute()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    /* …do something with `example`… */
    Ok(serde_wasm_bindgen::to_value(&response)?)
}

#[wasm_bindgen]
pub fn calc_get_token_in_by_token_out(val: JsValue) -> Result<JsValue, JsValue> {
    let request: GetTokenInByTokenOutRequest = serde_wasm_bindgen::from_value(val)?;
    let response = request
        .execute()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    /* …do something with `example`… */
    Ok(serde_wasm_bindgen::to_value(&response)?)
}

#[wasm_bindgen]
pub fn calc_get_token_out_by_token_in(val: JsValue) -> Result<JsValue, JsValue> {
    let request: GetTokenOutByTokenInRequest = serde_wasm_bindgen::from_value(val)?;
    let response = request
        .execute()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    /* …do something with `example`… */
    Ok(serde_wasm_bindgen::to_value(&response)?)
}
