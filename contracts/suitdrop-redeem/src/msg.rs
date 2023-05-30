use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint64;

use crate::state::Redemption;

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub redemption_denom: String,
    pub nft_code_id: Uint64,
    pub nft_name: String,
    pub nft_symbol: String,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    Redeem { proof: String },
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(RedemptionsResponse)]
    Redemptions {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(Redemption)]
    Redemption {
        id: Option<String>,
        proof: Option<String>,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct RedemptionsResponse {
    pub redemptions: Vec<Redemption>,
}

#[cw_serde]
pub struct RedemptionResponse {
    pub redemption: Redemption,
}
