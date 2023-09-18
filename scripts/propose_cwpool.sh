#!/bin/bash

source .env

sh ./scripts/config.sh

osmosisd query txs \
  --events 'message.action=/osmosis.cosmwasmpool.v1beta1.MsgCreateCosmWasmPool' -o json | jq 

osmosisd q gov params --node $OSMOSIS_NODE --chain-id $OSMOSIS_CHAIN_ID

OSMOSIS_KEY_NAME="suitdroptestnet"

# subtract 1 from deposit so someone else can start voting period
DEPOSIT_AMOUNT=$(echo "100000000" | bc)

osmosisd tx gov submit-proposal upload-code-id-and-whitelist ./artifacts/cw_bonding_pool.wasm \
  --title "$(jq -r .title ./scripts/propose_cwpool.proposal.json)" \
  --description "$(jq -r .description ./scripts/propose_cwpool.proposal.json)" \
  --deposit '125000000uosmo' \
  --from $OSMOSIS_KEY_NAME \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 0.025uosmo \
  --broadcast-mode block \
  --chain-id $OSMOSIS_CHAIN_ID \
  --node $OSMOSIS_NODE \
  --output json \
  --yes