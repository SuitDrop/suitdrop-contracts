use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;

/// Message type for `instantiate` entry_point
pub type InstantiateMsg = cw721_base::InstantiateMsg;
/// Message type for `execute` entry_point

// alias cw721_base::ExecuteMsg<cw721_base::Extension, Empty> as ExecuteMsg

pub type ExecuteMsg = cw721_base::ExecuteMsg<cw721_base::Extension, Empty>;

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
pub type QueryMsg = cw721_base::QueryMsg<Empty>;

// We define a custom struct for each query response
// #[cw_serde]
// pub struct YourQueryResponse {}
