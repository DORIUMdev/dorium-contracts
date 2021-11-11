# default home is ~/.wasmd
# if you want to setup multiple apps on your local make sure to change this value
APP_HOME="./wasmddata"
RPC="tcp://0.0.0.0:26657"
CHAIN_ID="localnet"

rm -rf $APP_HOME
# initialize wasmd configuration files
wasmd init localnet --chain-id ${CHAIN_ID} --home ${APP_HOME}

# add minimum gas prices config to app configuration file
sed -i -r 's/minimum-gas-prices = ""/minimum-gas-prices = "0.001udor"/' ${APP_HOME}/config/app.toml

# enable 1317 API
perl -0777 -i.original -pe 's/# Enable defines if the API server should be enabled.\nenable = false/# Enable defines if the API server should be enabled.\nenable = true/igs' ${APP_HOME}/config/app.toml

# resolve CORS policy blocking during request to the chain by the Keplr API
sed -i -r 's/cors_allowed_origins = \[\]/cors_allowed_origins = \["*"\]/' ${APP_HOME}/config/config.toml

# add your wallet addresses to genesis
wasmd add-genesis-account $(wasmd keys show -a main --keyring-backend=test) 10000000000udor,10000000000stake --home ${APP_HOME}
wasmd add-genesis-account $(wasmd keys show -a validator --keyring-backend=test) 10000000000udor,10000000000stake --home ${APP_HOME}

# add fred's address as validator's address
wasmd gentx validator 1000000000stake --home ${APP_HOME} --chain-id ${CHAIN_ID} --keyring-backend=test --keyring-dir=~/.wasmd

# collect gentxs to genesis
wasmd collect-gentxs --home ${APP_HOME}

# validate the genesis file
wasmd validate-genesis --home ${APP_HOME}

# run the node
wasmd start --home ${APP_HOME} --rpc.laddr ${RPC}
