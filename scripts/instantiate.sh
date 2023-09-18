#!/bin/bash

source .env

# osmosisd query txs \
#   --events 'message.action=/osmosis.cosmwasmpool.v1beta1.MsgCreateCosmWasmPool' -o json | jq 

OSMOSIS_KEY_NAME="suitdroptestnet"

CW_BONDING_INSTANTIATE_MSG='{
  "supply_subdenom": "ushirt",
  "supply_decimals": 6,
  "reserve_denom": "uosmo",
  "reserve_decimals": 6,
  "max_supply": "500000000",
  "test_mode": true,
  "curve_type": {
    "linear": {
      "slope": "1",
      "scale": 1
    }
  }
}'

osmosisd config chain-id $OSMOSIS_CHAIN_ID
osmosisd config node $OSMOSIS_NODE

KEY_BECH32=$(osmosisd keys show $OSMOSIS_KEY_NAME -a --keyring-backend test)


# echo "Signing message..."

# # sign message 
# osmosisd tx sign ./create_cw_pool_msg.json \
#   --from $OSMOSIS_KEY_NAME \
#   --offline \
#   --chain-id $OSMOSIS_CHAIN_ID \
#   --node $OSMOSIS_NODE \
#   --account-number $(osmosisd query account $KEY_BECH32 --chain-id $OSMOSIS_CHAIN_ID --node $OSMOSIS_NODE --output json | jq -r '.account_number') \
#   --keyring-backend test \
#   --sequence $(osmosisd query account $KEY_BECH32 --chain-id $OSMOSIS_CHAIN_ID --node $OSMOSIS_NODE --output json | jq -r '.sequence') \
#   --output json \
#   --output-document ./create_cw_pool_msg_signed.json 


# # Instantiate contract
INSTANTIATE_OUT=$(osmosisd tx wasm instantiate \
  $(cat artifacts/code_id.cw_bonding_pool-aarch64.wasm.txt) \
  "$CW_BONDING_INSTANTIATE_MSG" \
  --from $OSMOSIS_KEY_NAME \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 0.025uosmo \
  --broadcast-mode block \
  --chain-id $OSMOSIS_CHAIN_ID \
  --node $OSMOSIS_NODE \
  --keyring-backend test \
  --label "cw_bonding_pool" \
  --output json \
  --admin $KEY_BECH32 \
  --yes \
  --output json)

CONTRACT_ADDRESS=$(echo $INSTANTIATE_OUT | jq -r '.logs[] | .events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
DENOM=$(echo $INSTANTIATE_OUT | jq -r '.logs[] | .events[] | select(.type == "create_denom") | .attributes[] | select(.key == "new_token_denom") | .value')

echo "CONTRACT_ADDRESS: $CONTRACT_ADDRESS"
echo "DENOM: $DENOM"

sh ./scripts/instantiate_redeem.sh $CONTRACT_ADDRESS $DENOM