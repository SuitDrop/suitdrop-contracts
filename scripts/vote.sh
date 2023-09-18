source .env 
source ./scripts/config.sh

# get id of most recent proposal 
PROPOSAL_ID=$(osmosisd query gov proposals --limit 1 --reverse --node $OSMOSIS_NODE --chain-id $OSMOSIS_CHAIN_ID --output json | jq -r .proposals[0].proposal_id)

echo "PROPOSAL_ID: $PROPOSAL_ID"

osmosisd tx gov vote $PROPOSAL_ID yes \
  --from val \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 0.025uosmo \
  --broadcast-mode block \
  --chain-id $OSMOSIS_CHAIN_ID \
  --node $OSMOSIS_NODE \
  --output json \
  --yes