use cw_bonding_pool::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cw_orch::interface;
use cw_orch::prelude::*;

#[interface(
    cw_bonding_pool::msg::InstantiateMsg,
    cw_bonding_pool::msg::ExecuteMsg,
    cw_bonding_pool::msg::QueryMsg,
    cw_bonding_pool::msg::MigrateMsg
)]
pub struct CwBondingPool;

// Implement the Uploadable trait so it can be uploaded to the mock.
impl<Chain: CwEnv> Uploadable for CwBondingPool<Chain> {
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                cw_bonding_pool::contract::execute,
                cw_bonding_pool::contract::instantiate,
                cw_bonding_pool::contract::query,
            )
            .with_migrate(cw_bonding_pool::contract::migrate),
        )
    }
}

#[cfg(test)]
mod test {
    use cw_bonding_pool::msg::CurveType;

    use super::*;

    #[test]
    fn test_upload() {
        // ## Environment setup ##

        let sender = Addr::unchecked("sender");
        // Create a new mock chain (backed by cw-multi-test)
        let chain = Mock::new(&sender);

        // Create a new Cw20 interface
        let cw_bonding_pool: CwBondingPool<Mock> = CwBondingPool::new("my_token", chain);

        // Upload the contract
        cw_bonding_pool.upload().unwrap();

        // Instantiate a CW20 token
        let cw20_init_msg = cw_bonding_pool::msg::InstantiateMsg {
            supply_subdenom: "ushirt".to_string(),
            supply_decimals: 6,
            max_supply: 500_000_000u128.into(),
            reserve_denom: "uosmo".to_string(),
            reserve_decimals: 6,
            curve_type: CurveType::Linear {
                slope: 1u128.into(),
                scale: 1u32,
            },
        };
        cw_bonding_pool
            .instantiate(&cw20_init_msg, None, None)
            .unwrap();

        // Query the balance
        let balance: BalanceResponse = cw_bonding_pool
            .query(&QueryMsg::Balance {
                address: sender.to_string(),
            })
            .unwrap();

        assert_eq!(balance.balance.u128(), 10u128);
    }
}
