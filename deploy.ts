require('dotenv').config()
import * as fs from 'fs';
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

async function main() {
	const address_prefix = "wasm"

	const wallet_main = await DirectSecp256k1HdWallet.fromMnemonic(process.env.MNE_MAIN, { prefix: address_prefix });
	const accounts = await wallet_main.getAccounts()
	console.log("Wallet Accounts:", accounts)
	const options = { prefix: address_prefix };
	const client_main = await SigningCosmWasmClient.connectWithSigner("http://localhost:26657", wallet_main, options);

	const contract = fs.readFileSync('/home/shinichi/source/work/cosmos/cosmwasm-examples/escrow/target/wasm32-unknown-unknown/release/cw_escrow.wasm')
	const up = await client_main.upload(accounts[0].address, contract);
	console.log(up);

}
main()
