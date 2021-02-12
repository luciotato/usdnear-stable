set -e
NETWORK=testnet
OWNER=lucio.$NETWORK
OPERATOR=$OWNER
MASTER_ACC=stable.$NETWORK
CONTRACT_ACC=usdnear.$MASTER_ACC

usdnear --cliconf -c $CONTRACT_ACC -acc $OWNER

export NODE_ENV=$NETWORK

## delete acc
echo "Delete $CONTRACT_ACC? are you sure? Ctrl-C to cancel"
read input
near delete $CONTRACT_ACC $MASTER_ACC
near create-account $CONTRACT_ACC --masterAccount $MASTER_ACC
usdnear deploy ./res/usdnear.wasm  --accountId $MASTER_ACC
near call $CONTRACT_ACC new "{\"owner_account_id\":\"$OWNER\", \"treasury_account_id\":\"treasury.$CONTRACT_ACC\", \"operator_account_id\":\"$OPERATOR\",\"current_stnear_price\":\"3310000000000000000000000\"}" --accountId $MASTER_ACC
# set params@stbl set_params
usdnear set_params
usdnear set_price 4.1

## redeploy code only
#usdnear deploy ./res/usdnear.wasm  --accountId $MASTER_ACC

#save last deployment  (to be able to recover state/tokens)
#cp ./res/usdnear.wasm ./res/usdnear.`date +%F.%T`.wasm
#date +%F.%T
