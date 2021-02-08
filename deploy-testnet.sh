set -e
NETWORK=testnet
OWNER=lucio.$NETWORK
OPERATOR=$OWNER
MASTER_ACC=stable.$NETWORK
CONTRACT_ACC=usdnear.$MASTER_ACC

stbl --cliconf -c $CONTRACT_ACC -acc $OWNER

export NODE_ENV=$NETWORK

## delete acc
#echo "Delete $CONTRACT_ACC? are you sure? Ctrl-C to cancel"
#read input
#near delete $CONTRACT_ACC $MASTER_ACC
#near create-account $CONTRACT_ACC --masterAccount $MASTER_ACC
#stbl deploy ./res/divpool.wasm
#stbl new { owner_account_id:$OWNER, treasury_account_id:treasury.$CONTRACT_ACC, operator_account_id:$OPERATOR } --accountId $MASTER_ACC
## set params@stbl set_params
#stbl default_pools_testnet

## redeploy code only
stbl deploy ./res/usdnear.wasm  --accountId $MASTER_ACC

