# import .env
source .env

OSMOSIS_KEY_NAME="suitdroptestnet"

# configure keys
echo "$TEST_MNEMONIC" | osmosisd keys add "$OSMOSIS_KEY_NAME" --keyring-backend test --recover

# var
TAG=$1

# assert valid tag provided. exit with message if not
if [ -z "$TAG" ]
then
  echo "No tag provided. Please provide a valid tag."
  exit 1
fi

# download latest compiled release artifacts from github via curl
# e.g. https://github.com/SuitDrop/suitdrop-contracts/releases/download/v0.0.7/cosmwasm-artifacts.tar.gz
curl -LO "https://github.com/SuitDrop/suitdrop-contracts/releases/download/$TAG/cosmwasm-artifacts.tar.gz"
echo "https://github.com/SuitDrop/suitdrop-contracts/releases/download/$TAG/cosmwasm-artifacts.tar.gz"

# extract tarball
tar -xvf cosmwasm-artifacts.tar.gz

# iteratively deploy each artifact to osmosis testnet via osmosisd tx wasm store

I=0
# loop 
for file in ./artifacts/*.wasm; do
    # increment counter
    I=$((I+1))
    echo "Deploying $file"
    # get code id
    TX_RESPONSE=$(osmosisd tx wasm store $file \
      --from $OSMOSIS_KEY_NAME \
      --gas auto \
      --gas-adjustment 1.5 \
      --gas-prices 0.025uosmo \
      --broadcast-mode block \
      --chain-id $OSMOSIS_CHAIN_ID \
      --node $OSMOSIS_NODE \
      --keyring-backend test \
      --output json \
      --yes)
    # write response to file
    echo $TX_RESPONSE > ./artifacts/tx_response.$I.json
    CODE_ID=$(echo $TX_RESPONSE | jq -r '.data.code_info.code_id')

    # get code hash
    CODE_HASH=$(osmosisd q wasm code $CODE_ID | jq -r '.data.code_info.code_hash')
    echo "Code ID: $CODE_ID, Code Hash: $CODE_HASH"
done
