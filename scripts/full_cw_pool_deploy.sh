beaker wasm build -a && \
mv artifacts/cw_bonding_pool-aarch64.wasm artifacts/cw_bonding_pool.wasm && \
sh ./scripts/propose_cwpool.sh && \
sh ./scripts/vote.sh

# poll for latest proposal to pass in loop with sleep

# get id of most recent proposal
PROPOSAL_ID=$(osmosisd query gov proposals --limit 1 --reverse --output json | jq -r .proposals[0].proposal_id)

echo "PROPOSAL_ID: $PROPOSAL_ID"

# wait for proposal to pass
while true; do
  # get proposal status
  PROPOSAL_STATUS=$(osmosisd query gov proposal $PROPOSAL_ID --output json | jq -r .status)
  echo "PROPOSAL_STATUS: $PROPOSAL_STATUS"
  # if proposal status is passed, break
  if [ "$PROPOSAL_STATUS" = "PROPOSAL_STATUS_PASSED" ]; then
    break
  fi
  # sleep for 5 seconds
  sleep 5
done

# get id of most recently whitelisted cw pool from params
CW_POOL_CODE_ID=$(osmosisd q cosmwasmpool params --output json | jq -r '.params.code_id_whitelist | last');

echo "CW_POOL_CODE_ID: $CW_POOL_CODE_ID"

sh ./scripts/instantiate_cosmwasmpool.sh $CW_POOL_CODE_ID

POOL_ID=$(osmosisd q poolmanager all-pools --output json | jq -r '.pools | last | .pool_id');

echo "POOL_ID: $POOL_ID"

POOL_CONTRACT_ADDRESS=$(osmosisd q poolmanager pool $POOL_ID --output json | jq -r '.pool.contract_address');

echo "POOL_CONTRACT_ADDRESS: $POOL_CONTRACT_ADDRESS"

MINT_DENOM=$(osmosisd q tokenfactory denoms-from-creator $POOL_CONTRACT_ADDRESS --output json | jq -r '.denoms | first')

echo "MINT_DENOM: $MINT_DENOM"

osmosisd tx poolmanager swap-exact-amount-in 10000uosmo $POOL_ID --from val --swap-route-denoms "$MINT_DENOM" --swap-route-pool-ids $POOL_ID

osmosisd q wasm cs smart $POOL_CONTRACT_ADDRESS '{ "get_total_pool_liquidity": {} }'