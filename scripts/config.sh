# import .env
source .env 


osmosisd keys delete suitdroptestnet --yes

echo "$SUITDROP_MNEMONIC" | osmosisd keys add "$OSMOSIS_KEY_NAME" --recover

osmosisd q bank balances $(osmosisd keys show $OSMOSIS_KEY_NAME -a --keyring-backend test) --node $OSMOSIS_NODE --chain-id $OSMOSIS_CHAIN_ID

osmosisd config node $OSMOSIS_NODE

osmosisd config chain-id $OSMOSIS_CHAIN_ID

osmosisd config gas-prices 0.025uosmo

osmosisd config gas auto

osmosisd config broadcast-mode block

osmosisd config gas-adjustment 1.5