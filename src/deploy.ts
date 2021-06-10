import { config } from 'dotenv';
import * as fs from 'fs';
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

config();

const { ERC20_CONTRACT, ESCROW_CONTRACT, DORIUM_PROPOSAL_CONTRACT, MNEMONIC_MAIN, RPC_ENDPOINT } = process.env;
const options = { prefix: "wasm" };
const ERC20Contract = fs.readFileSync(ERC20_CONTRACT)
const EscrowContract = fs.readFileSync(ESCROW_CONTRACT);
const ProposalContract = fs.readFileSync(DORIUM_PROPOSAL_CONTRACT);

async function getWalletData() {
	const wallet_main = await DirectSecp256k1HdWallet.fromMnemonic(MNEMONIC_MAIN, options);

	return wallet_main;
}

async function getWalletAccount() {
	const walletData = await getWalletData();
	const [mainAccount] = await walletData.getAccounts();

	return mainAccount;
}

async function instantiateCW20() {
	const account = await getWalletAccount();
	const wallet = await getWalletData();
	const client = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, wallet, options);
	const initMsg = {
		name: "Dorium Value Token",
		symbol: "TREE",
		decimals: 2,
		initial_balances: [
			{ address: "wasm1ryuawewrukex42yh2kpydtpdh90ex096kaajek", amount: "3040000000000"}, // number of trees in the world according to Google
		],
		mint: {
			minter: "wasm1ryuawewrukex42yh2kpydtpdh90ex096kaajek"
		}
	}
	const contractData = await client.upload(account.address, ERC20Contract)
	const contractAddress = await client.instantiate(account.address, contractData.codeId, initMsg, "creating the cw20 token")
	console.log('CW20 Value Token Uploaded Contract', contractData);
	console.log('CW20 Value Token Instantiated Contract', contractAddress);
}

async function uploadEscrow() {
	const account = await getWalletAccount();
	const wallet = await getWalletData();
	const client = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, wallet, options);
	const contractData = await client.upload(account.address, EscrowContract)
	console.log('Escrow', contractData);
}


async function uploadProposal() {
	const account = await getWalletAccount();
	const wallet = await getWalletData();
	const client = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, wallet, options);
	const contractData = await client.upload(account.address, ProposalContract)
	console.log('DORCP', contractData);
}

export async function main() {
	try {
		await instantiateCW20();
		await uploadEscrow();
		await uploadProposal();
	} catch (e) {
		throw e;
	}
}

