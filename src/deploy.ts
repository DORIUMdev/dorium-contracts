import { config } from 'dotenv';
import * as fs from 'fs';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { ExecuteResult, InstantiateResult, SigningCosmWasmClient, UploadResult } from '@cosmjs/cosmwasm-stargate';
import { Coin } from '@cosmjs/proto-signing/build/codec/cosmos/base/v1beta1/coin';

config();

const { ERC20_CONTRACT, DORIUM_PROPOSAL_CONTRACT, MNEMONIC_MAIN, RPC_ENDPOINT } = process.env;
const options = { prefix: "wasm" };
const ERC20Contract = fs.readFileSync(ERC20_CONTRACT)
const ProposalContract = fs.readFileSync(DORIUM_PROPOSAL_CONTRACT);

async function getWalletData() {
	return await DirectSecp256k1HdWallet.fromMnemonic(MNEMONIC_MAIN, options);
}

async function getWalletAccount() {
	const walletData = await getWalletData();
	const [mainAccount] = await walletData.getAccounts();

	return mainAccount;
}

async function uploadContracts(account: AccountData, wallet: DirectSecp256k1HdWallet, client: SigningCosmWasmClient) {
	const con_cw20 = await client.upload(account.address, ERC20Contract);
	console.log("CW20 Uploaded Contract", con_cw20);
	const con_dorcp = await client.upload(account.address, ProposalContract);
	console.log("DORCP Uploaded Contract", con_dorcp);
	const contracts = {
		cw20: {codeId: con_cw20.codeId, transactionHash: con_cw20.transactionHash},
		dorcp: {codeId: con_dorcp.codeId, transactionHash: con_dorcp.transactionHash},
	}
	return contracts
}

export async function uploadContracts2() {
	const account = await getWalletAccount();
	const wallet = await getWalletData();
	const client = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, wallet, options);

	const contracts = await uploadContracts(account, wallet, client);
	fs.writeFileSync("contracts.json", JSON.stringify(contracts))
}

async function instantiateCW20(contractData: UploadResult, account: AccountData, wallet: DirectSecp256k1HdWallet, client: SigningCosmWasmClient) {
	const initMsg = {
		name: 'Dorium Value Token',
		symbol: 'TREE',
		decimals: 2,
		initial_balances: [
			{ address: 'wasm1ryuawewrukex42yh2kpydtpdh90ex096kaajek', amount: '3040000000000' }, // number of trees in the world according to Google
		],
		mint: {
			minter: 'wasm1ryuawewrukex42yh2kpydtpdh90ex096kaajek',
		},
	};

	const instanceData = await client.instantiate(account.address, contractData.codeId, initMsg, "instantiating the DORCP contract");
	return instanceData
}

async function instantiateDoriumCommunityProposal(contractData: UploadResult, account: AccountData, wallet: DirectSecp256k1HdWallet, client: SigningCosmWasmClient): Promise<[InstantiateResult, ExecuteResult]> {
	const instantiateData = await client.instantiate(
		account.address,
		contractData.codeId,
		{},
		'creating the cw20 token'
	);
	const contractAddress = instantiateData.contractAddress;

	const createMsg = {
		create:{
		description: "Test Description",
		id: "dorcp-test1",
		proposer: account.address,
		source: account.address,
		validators: [account.address],
	}}

	const funds = Coin.fromJSON({denom: "atom", amount: "100000"})
	const createData = await client.execute(account.address, contractAddress, createMsg, "just deploying a DORCP here", [funds])
	return [instantiateData, createData]
}


export async function main() {
	try {
		const account = await getWalletAccount();
		const wallet = await getWalletData();
		const client = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, wallet, options);

		var con = JSON.parse(fs.readFileSync("contracts.json").toString());
		const inst_cw20 = await instantiateCW20(con.cw20, account, wallet, client);
		console.log("CW20 Instantiated", inst_cw20);
		const inst_dorcp = await instantiateDoriumCommunityProposal(con.dorcp, account, wallet, client);
		console.log("DORCP Instantiated", inst_dorcp);
	} catch (e) {
		throw e;
	}
}
