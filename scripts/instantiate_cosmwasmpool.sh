#!/bin/bash

source .env

sh ./scripts/config.sh

osmosisd query txs \
  --events 'message.action=/osmosis.cosmwasmpool.v1beta1.MsgCreateCosmWasmPool' -o json | jq 

OSMOSIS_KEY_NAME="suitdroptestnet"

# osmosisd tx gov submit-proposal upload-code-id-and-whitelist ./artifacts/cw_bonding_pool.wasm \
#   --from $OSMOSIS_KEY_NAME \
#   --gas auto \
#   --gas-adjustment 1.5 \
#   --gas-prices 0.025uosmo \
#   --broadcast-mode block \
#   --chain-id $OSMOSIS_CHAIN_ID \
#   --node $OSMOSIS_NODE \
#   --keyring-backend test \
#   --output json \
#   --yes

osmosisd q bank balances $(osmosisd keys show $OSMOSIS_KEY_NAME -a --keyring-backend test) --node $OSMOSIS_NODE --chain-id $OSMOSIS_CHAIN_ID

# Instantiate cw-bonding-pool
CW_BONDING_INSTANTIATE_MSG='{
  "supply_subdenom": "ushirt",
  "reserve_denom": "uosmo",
  "max_supply": "500000000",
  "curve_type": {
    "linear": {
      "slope": "1",
      "scale": 1
    }
  },
  "reserve_decimals": 6,
  "supply_decimals": 6
}';

# code id is $1 if passed in, otherwise use 2 as default 
CODE_ID=${1:-2}

echo "code id: $CODE_ID"

# osmosisd tx cosmwasmpool create-pool $CODE_ID \
#   --from $KEY_BECH32 \
#   --chain-id $OSMOSIS_CHAIN_ID \
#   --node $OSMOSIS_NODE \
#   --gas auto \
#   --gas-adjustment 1.5 \
#   --gas-prices 0.025uosmo \
#   --broadcast-mode block \
#   --yes 

BASE_64_CW_BONDING_INSTANTIATE_MSG=$(echo $CW_BONDING_INSTANTIATE_MSG | base64)


KEY_BECH32=$(osmosisd keys show $OSMOSIS_KEY_NAME -a --keyring-backend test)

echo $KEY_BECH32CREATE_COSMWASM_POOL_MSG 
  # "@type": "/osmosis.cosmwasmpool.v1beta1.MsgCreateCosmWasmPool",
CREATE_COSMWASM_POOL_MSG='{
  "body": {
    "messages": [
      {
        "@type": "/osmosis.cosmwasmpool.v1beta1.MsgCreateCosmWasmPool",
        "sender": "osmo1cyyzpxplxdzkeea7kwsydadg87357qnahakaks",
        "code_id": '$CODE_ID',
        "instantiate_msg": "'$BASE_64_CW_BONDING_INSTANTIATE_MSG'"
      }
    ],
    "memo": "",
    "timeout_height": "0",
    "extension_options": [],
    "non_critical_extension_options": []
  },
  "auth_info": {
    "signer_infos": [],
    "fee": {
      "amount": [
        {
          "denom": "uosmo",
          "amount": "23013"
        }
      ],
      "gas_limit": "9204928",
      "payer": "",
      "granter": ""
    }
  },
  "signatures": []
}'


echo $CREATE_COSMWASM_POOL_MSG

echo "Writing to file..."

# write to file
echo $CREATE_COSMWASM_POOL_MSG > ./create_cw_pool_msg.json

echo "Signing message..."

# sign message 
osmosisd tx sign ./create_cw_pool_msg.json \
  --from $OSMOSIS_KEY_NAME \
  --offline \
  --chain-id $OSMOSIS_CHAIN_ID \
  --node $OSMOSIS_NODE \
  --account-number $(osmosisd query account $KEY_BECH32 --chain-id $OSMOSIS_CHAIN_ID --node $OSMOSIS_NODE --output json | jq -r '.account_number') \
  --keyring-backend test \
  --sequence $(osmosisd query account $KEY_BECH32 --chain-id $OSMOSIS_CHAIN_ID --node $OSMOSIS_NODE --output json | jq -r '.sequence') \
  --output json \
  --output-document ./create_cw_pool_msg_signed.json 

echo "Broadcasting message..."

# broadcast message
osmosisd tx broadcast ./create_cw_pool_msg_signed.json \
  --chain-id $OSMOSIS_CHAIN_ID \
  --node $OSMOSIS_NODE \
  --keyring-backend test \
  --broadcast-mode block \
  --output json | jq > ./create_cw_pool_msg_broadcast.json

cat ./create_cw_pool_msg_broadcast.json
# echo "Done!"


# osmosisd tx gamm create-pool \
#   --pool-file ./samplepool.json \
#   --from $KEY_BECH32 \
#   --chain-id $OSMOSIS_CHAIN_ID \
#   --node $OSMOSIS_NODE \
#   --gas auto \
#   --sequence $(osmosisd query account $KEY_BECH32 --chain-id $OSMOSIS_CHAIN_ID --node $OSMOSIS_NODE --output json | jq -r '.sequence') \
#   --gas-adjustment 1.5 \
#   --gas-prices 0.025uosmo \
#   --generate-only > ./create_pool_msg.json
