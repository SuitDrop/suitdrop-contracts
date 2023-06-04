use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};
use cw_storage_macro::index_list;
use cw_storage_plus::{IndexedMap, Item, UniqueIndex};

// use `cw_storage_plus` to create ORM-like interface to storage
// see: https://crates.io/crates/cw-storage-plus
#[cw_serde]
pub struct Redemption {
    pub id: String,
    pub proof: String,
    pub redeemer: String,
}

#[index_list(Redemption)]
pub struct RedemptionIndexes<'a> {
    pub proof: UniqueIndex<'a, String, Redemption, String>,
}

pub struct RedemptionState<'a> {
    pub redemptions: IndexedMap<'a, String, Redemption, RedemptionIndexes<'a>>,
}

impl Default for RedemptionState<'_> {
    fn default() -> Self {
        Self {
            redemptions: IndexedMap::new(
                "redemptions",
                RedemptionIndexes {
                    proof: UniqueIndex::new(
                        |redemption| redemption.proof.clone(),
                        "redemptions_proofs",
                    ),
                },
            ),
        }
    }
}

pub const REDEMPTION_INCREMENT: Item<Uint64> = Item::new("redemption_increment");

pub const REDEMPTION_DENOM: Item<String> = Item::new("redemption_denom");

pub const COST_PER_UNIT: Item<Uint128> = Item::new("cost_per_unit");

pub const NFT_CONTRACT: Item<Addr> = Item::new("nft_contract");

pub const BONDING_CONTRACT: Item<Addr> = Item::new("bonding_contract");

pub const NFT_RECEIPT_TOKEN_URI: Item<String> = Item::new("nft_receipt_token_uri");
