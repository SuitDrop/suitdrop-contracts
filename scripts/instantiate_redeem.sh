#!/bin/bash

source .env


OSMOSIS_KEY_NAME="suitdroptestnet"
#  &suitdrop_redeem::msg::InstantiateMsg {
#             redemption_denom: "ushirt".to_string(),
#             nft_code_id: cw721_code_id.into(),
#             nft_name: "Shirt NFT".to_string(),
#             nft_symbol: "SHIRT".to_string(),
#             cost_per_unit: Uint128::from(1_000_000u128),
#             // filler
#             bonding_contract_addr: mock_cw_bonding_addr.to_string(),
#             nft_receipt_token_uri: "https://example.com".to_string(),
#         },

BONDING_CONTRACT_ADDR=$1
ORDER_VOUCHER_DENOM=$2

INSTANTIATE_MSG='{
  "redemption_denom": "'$ORDER_VOUCHER_DENOM'",
  "nft_code_id": "'$(cat artifacts/code_id.cw721_suit.wasm.txt)'",
  "nft_name": "Shirt NFT",
  "nft_symbol": "SHIRTREDEEM",
  "cost_per_unit": "1000000",
  "bonding_contract_addr": "'$BONDING_CONTRACT_ADDR'",
  "nft_receipt_token_uri": "ipfs://QmPjCme3eFgNia4BuRV4Y19A8eQesBSPrVqYDc5cXWxnJa/1.json"
}'

# osmo1ft5w60ga0z9a2qskjy3ecpqa0z2s5ukpt4s03y0hstk0s0kj20uq6yrgts

KEY_BECH32=$(osmosisd keys show $OSMOSIS_KEY_NAME -a --keyring-backend test)

# # Instantiate contract
INSTANTIATE_OUT=$(osmosisd tx wasm instantiate \
  $(cat artifacts/code_id.suitdrop_redeem.wasm.txt) \
  "$INSTANTIATE_MSG" \
  --from $OSMOSIS_KEY_NAME \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 0.025uosmo \
  --broadcast-mode block \
  --chain-id $OSMOSIS_CHAIN_ID \
  --node $OSMOSIS_NODE \
  --keyring-backend test \
  --label "merchdrop_redeem" \
  --output json \
  --admin $KEY_BECH32 \
  --yes)

REDEEM_CONTRACT_ADDRESS=$(echo "$INSTANTIATE_OUT" | jq -r '[.logs | .[] | .events | .[] | select(.type=="instantiate").attributes[] | select(.key=="_contract_address").value] | .[0]')
NFT_CONTRACT_ADDRESS=$(echo "$INSTANTIATE_OUT" | jq -r '[.logs | .[] | .events | .[] | select(.type=="instantiate").attributes[] | select(.key=="_contract_address").value] | .[1]')

echo "Redeem contract address: $REDEEM_CONTRACT_ADDRESS"
echo "NFT contract address: $NFT_CONTRACT_ADDRESS"