# How to Use
1. install https://github.com/CosmWasm/wasmd.git, checkout branch v0.16.0 or v0.17.0 (these contracts are written with cosmwasm 0.14, which requires these versions)
2. `cp .env.example .env`, run `wasmd keys mnemonic` twice, copy the output into MNEMONIC_* in `.env`
3. `./wasmdkeys.sh` - this runs `wasmd keys add` for the accounts main and validator
4. `./wasmdstart.sh` to start the blockchain (data is stored in `./wasmddata`, which  be deleted every time you run this)
5. `npm install`
6. `ts-node index.ts upload` to upload compiled .wasm smart contracts under `./binaries` to the blockchain. The upload results are stored in `./contracts.json`
7. `ts-node index.ts deploy` to deploy the wasm smart contracts with some default settings (see `src/deploy.ts`)