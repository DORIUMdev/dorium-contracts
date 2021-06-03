import { config } from 'dotenv';
import * as fs from 'fs';
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

config();

const { ERC20_CONTRACT, ESCROW_CONTRACT, MNEMONIC_MAIN, RPC_ENDPOINT } = process.env;
const options = { prefix: "wasm" };
const ERC20Contract = fs.readFileSync(ERC20_CONTRACT)
const EscrowContract = fs.readFileSync(ESCROW_CONTRACT);

async function getWalletData() {
	const wallet_main = await DirectSecp256k1HdWallet.fromMnemonic(MNEMONIC_MAIN, options);
	return wallet_main;
}

async function getWalletAccount() {
	const walletData = await getWalletData();
	const [mainAccount] = await walletData.getAccounts();
	return mainAccount;
}

async function uploadERC20() {
	const account = await getWalletAccount();
	const wallet = await getWalletData();
	const client_main = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, wallet, options);
	const contractData = await client_main.upload(account.address, ERC20Contract)
	console.log('ERC20', contractData);

}

async function uploadEscrow() {
	const account = await getWalletAccount();
	const wallet = await getWalletData();
	const client_main = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, wallet, options);
	const contractData = await client_main.upload(account.address, EscrowContract)
	console.log('Escrow', contractData);

}

export async function main() {
	try {
		await uploadERC20();
		// await uploadEscrow();
	} catch (e) {
		throw e;
	}
}

