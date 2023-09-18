# import .env
source .env

source ./scripts/config.sh

# var
# TAG=$1

# # assert valid tag provided. exit with message if not
# if [ -z "$TAG" ]
# then
#   echo "No tag provided. Please provide a valid tag."
#   exit 1
# fi

# download latest compiled release artifacts from github via curl
# e.g. https://github.com/SuitDrop/suitdrop-contracts/releases/download/v0.0.7/cosmwasm-artifacts.tar.gz
# curl -LO "https://github.com/SuitDrop/suitdrop-contracts/releases/download/$TAG/cosmwasm-artifacts.tar.gz"
# echo "https://github.com/SuitDrop/suitdrop-contracts/releases/download/$TAG/cosmwasm-artifacts.tar.gz"

# # extract tarball
# tar -xvf cosmwasm-artifacts.tar.gz

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
    # wait 6 seconds 
    sleep 6
    # write response to file
    FILE_NAME=$(basename $file)
    echo $TX_RESPONSE > ./artifacts/tx_response.$FILE_NAME.json
    CODE_ID=$(echo $TX_RESPONSE | jq -r '.logs[].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
    # write code id to file 
    # extract filename from path
    echo $CODE_ID > ./artifacts/code_id.$FILE_NAME.txt
    echo "$FILE_NAME CODE ID: $CODE_ID"
done
