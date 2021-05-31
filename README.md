# How to Use
1. install https://github.com/CosmWasm/wasmd.git, checkout branch v0.15.1
2. `cp .env.example .env`, run `wasmd keys mnemonic` twice, copy the output into MNEMONIC_* in `.env`
3. `./wasmdkeys.sh` - this runs `wasmd keys add` for the accounts main and validator
4. `./wasmdstart.sh` to start the blockchain (will be deleted every time you run this)
5. `npm install` and `ts-node src/deploy.ts` to deploy compiled .wasm smart contracts to the blockchain