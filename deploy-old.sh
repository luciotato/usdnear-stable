# deploy old version if new breaking changes cant read old state
set -e
NETWORK=testnet
OWNER=lucio.$NETWORK
OPERATOR=$OWNER
MASTER_ACC=stable.$NETWORK
CONTRACT_ACC=usdnear.$MASTER_ACC

usdnear --cliconf -c $CONTRACT_ACC -acc $OWNER

export NODE_ENV=$NETWORK

## redeploy code only
usdnear deploy $1  --accountId $MASTER_ACC

