#!/bin/bash

source .env

OSMOSIS_KEY_NAME="suitdroptestnet"

KEY_BECH32=$(osmosisd keys show $OSMOSIS_KEY_NAME -a --keyring-backend test)

MERKLE_ROOT="50a053def35526a1e0e00d990653dc02dde14ae4e58b003d66ee25481470be0c"

AIRDROP_INSTANTIATE_MSG='{
  "owner": "'$KEY_BECH32'",
  "native_token": "factory/osmo1j6w95sl6f2n2cr4txc9kq7rcstt40dy67ql6fhxvuw6f2kjw0u4qgll72l/ushirt"
}'

osmosisd config chain-id $OSMOSIS_CHAIN_ID
osmosisd config node $OSMOSIS_NODE


# # Instantiate contract
INSTANTIATE_OUT=$(osmosisd tx wasm instantiate \
  $(cat artifacts/code_id.cw20_merkle_airdrop.wasm.txt) \
  "$AIRDROP_INSTANTIATE_MSG" \
  --from $OSMOSIS_KEY_NAME \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 0.025uosmo \
  --broadcast-mode block \
  --chain-id $OSMOSIS_CHAIN_ID \
  --node $OSMOSIS_NODE \
  --keyring-backend test \
  --label "cw20_merkle_airdrop" \
  --output json \
  --admin $KEY_BECH32 \
  --yes \
  --output json)

CONTRACT_ADDRESS=$(echo $INSTANTIATE_OUT | jq -r '.logs[] | .events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')

echo "CONTRACT_ADDRESS: $CONTRACT_ADDRESS"

# register_merkle_root
EXECUTE_OUT=$(osmosisd tx wasm execute $CONTRACT_ADDRESS \
  '{
    "register_merkle_root": {
      "merkle_root": "'$MERKLE_ROOT'"
    }
  }' \
  --from $OSMOSIS_KEY_NAME \
  --yes \
  --output json);
