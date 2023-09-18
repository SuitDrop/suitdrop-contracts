source .env 

source ./scripts/config.sh

echo "hello"

osmosisd tx gov submit-proposal param-change ./tf_fee_prop.json \
  --from $OSMOSIS_KEY_NAME \
  --gas auto \
  --gas-adjustment 1.5 \
  --gas-prices 0.025uosmo \
  --broadcast-mode block \
  --chain-id $OSMOSIS_CHAIN_ID \
  --node $OSMOSIS_NODE \
  --output json \
  --yes

