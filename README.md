# Dorium Contracts
This repository contains the Dorium proposal and Value-SoBz Exchange Rust smart contracts, as well as tools to start the blockchain on which these smart contracts should be deployed.

Actual interaction with the smart contracts and the running blockchain is handled by code at https://github.com/apeunit/dorcp-helper

# How to Use
1. Build and install wasmd from https://github.com/CosmWasm/wasmd.git, checkout branch v0.20.0 (the cosmwasm smart contract library versions that these smart contracts rely on decide which wasmd version you should run)
2. Run `wasmd keys mnemonic` twice to generate two new account secrets. One of them is used as the blockchain validator, and one the other will be used to deploy the smart contracts from. Tell `wasmd` to remember these accounts by running `wasmd keys add main --keyring-backend=test --recover` and `wasmd keys add validator --keyring-backend=test --recover` respectively.
3. You may now run `wasmdstart.sh` to generate a new genesis file and start the blockchain.
4. Pull the repo https://github.com/apeunit/dorcp-helper and follow the instructions there. Paste the two mnemonics into the example `.env` file, which must reside in the `dorcp-helper` directory.
5. `wasmd.service` is a systemd unit file which you can copy somewhere (depends on your Linux distro) so that systemd can ensure that the wasmd is always running.