use cosmwasm_std::{coins, BankMsg, Coin, CosmosMsg, Uint128};
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgMint};

pub fn mint_or_send(
    supply_denom: String,
    spend_denom: String,
    amount: Uint128,
    sender: String,
    contract: String,
) -> CosmosMsg {
    if spend_denom == supply_denom {
        MsgMint {
            amount: osmosis_std::cosmwasm_to_proto_coins(coins(amount.u128(), spend_denom)).pop(),
            mint_to_address: sender,
            sender: contract,
        }
        .into()
    } else {
        BankMsg::Send {
            to_address: sender,
            amount: coins(amount.u128(), spend_denom),
        }
        .into()
    }
}

pub fn burn_or_receive(
    supply_denom: String,
    received_denom: String,
    amount: Uint128,
    sender: String,
    contract: String,
) -> Vec<CosmosMsg> {
    if received_denom == supply_denom {
        let msg = MsgBurn {
            burn_from_address: sender,
            sender: contract,
            amount: osmosis_std::cosmwasm_to_proto_coins(coins(amount.u128(), received_denom))
                .pop(),
        };
        vec![msg.into()]
    } else {
        vec![]
    }
}

pub fn create_coin_io_messages(
    token_in: Coin,
    token_out: Coin,
    supply_denom: String,
    sender: String,
    contract: String,
) -> Vec<CosmosMsg> {
    let mut msgs = burn_or_receive(
        supply_denom.clone(),
        token_in.denom,
        token_in.amount,
        sender.clone(),
        contract.clone(),
    );
    msgs.push(mint_or_send(
        supply_denom,
        token_out.denom,
        token_out.amount,
        sender,
        contract,
    ));
    msgs
}

#[cfg(test)]
pub mod tests {
    

    use cosmwasm_std::{coin};

    use super::*;

    #[test]
    fn test_create_coin_io_messages() {
        let token_in = Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1000000u128),
        };
        let token_out = Coin {
            denom: "uosmo".to_string(),
            amount: Uint128::from(1000000u128),
        };
        let supply_denom = "uosmo".to_string();
        let sender = "sender".to_string();
        let msgs = create_coin_io_messages(
            token_in,
            token_out,
            supply_denom,
            sender.clone(),
            sender.clone(),
        );
        println!("{:?}", msgs);
        assert_eq!(msgs.len(), 1);
        assert_eq!(
            msgs[0].clone(),
            MsgMint {
                amount: Some(coin(1000000u128, "uosmo").into()),
                mint_to_address: sender.clone(),
                sender: sender,
            }
            .into()
        );

        match msgs[0].clone() {
            CosmosMsg::Stargate { type_url: _, value: _ } => {}
            _ => panic!("Unexpected message type"),
        }
    }
}
