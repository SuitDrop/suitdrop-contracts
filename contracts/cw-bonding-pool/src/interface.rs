#![allow(unused)]

use cw_orch::{
    anyhow::Result,
    interface,
    prelude::{queriers::Node, *},
};

use crate::{
    contract::CONTRACT_NAME,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct CwBondingPoolContract;

impl<Chain: CwEnv> Uploadable for CwBondingPoolContract<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        let crate_path = env!("CARGO_MANIFEST_DIR");
        let wasm_path = format!("{}/../artifacts/{}", crate_path, "mock.wasm");

        WasmPath::new(wasm_path).unwrap()
    }
    // Return a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            )
            .with_migrate(crate::contract::migrate),
        )
    }
}
