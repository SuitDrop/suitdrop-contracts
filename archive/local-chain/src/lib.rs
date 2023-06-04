

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coin, Uint128};
    use cw_bonding_pool::msg::{CurveType, ExecuteMsg, GetTotalPoolLiquidityResponse};
    use cw_orch::{anyhow, prelude::*, tokio};
    use tokio::runtime::Runtime;

    /// Script that registers the first Account in abstract (our Account)
    #[test]
    pub fn main() -> anyhow::Result<()> {
        dotenv::dotenv().ok();
        env_logger::init();

        let rt = Runtime::new()?;
        let network = networks::LOCAL_JUNO;
        let chain = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(network)
            .build()?;

        let counter = CounterContract::new("counter_contract", chain);

        counter.upload()?;
        counter.instantiate(&InstantiateMsg { count: 0 }, None, None)?;
        Ok(())
    }
}
