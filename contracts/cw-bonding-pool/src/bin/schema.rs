use cosmwasm_schema::write_api;

use cw_bonding_pool::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        migrate: MigrateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
        sudo: SudoMsg
    }
}
