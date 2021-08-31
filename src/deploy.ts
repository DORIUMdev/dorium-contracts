import { config } from 'dotenv';
import * as fs from 'fs';
import * as util from 'util';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { toBase64, toUtf8 } from '@cosmjs/encoding';
import { ExecuteResult, InstantiateResult, SigningCosmWasmClient, UploadResult } from '@cosmjs/cosmwasm-stargate';
import { Coin } from '@cosmjs/proto-signing/build/codec/cosmos/base/v1beta1/coin';
import { CW20 } from "./cw20-base-helpers";
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

function readContractsJson() {
	var con = JSON.parse(fs.readFileSync("contracts.json").toString());
	return con
}

function writeContractsJson(obj: any) {
	fs.writeFileSync("contracts.json", JSON.stringify(obj, null, "\t"))
}

async function uploadContracts(account: AccountData, wallet: DirectSecp256k1HdWallet, client: SigningCosmWasmClient) {
	const con_cw20 = await client.upload(account.address, ERC20Contract);
	console.log("CW20 Uploaded Contract", con_cw20);
	const con_dorcp = await client.upload(account.address, ProposalContract);
	console.log("DORCP Uploaded Contract", con_dorcp);
	const contracts = {
		"contracts": {
			cw20: {codeId: con_cw20.codeId, transactionHash: con_cw20.transactionHash},
			dorcp: {codeId: con_dorcp.codeId, transactionHash: con_dorcp.transactionHash},
		},
	}
	return contracts
}

export async function uploadContracts2() {
	const account = await getWalletAccount();
	const wallet = await getWalletData();
	const client = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, wallet, options);

	const contracts = await uploadContracts(account, wallet, client);
	writeContractsJson(contracts)
}

async function instantiateValueToken(contractData: UploadResult, account: AccountData, wallet: DirectSecp256k1HdWallet, client: SigningCosmWasmClient) {
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

async function instantiateSobzToken(contractData: UploadResult, account: AccountData, wallet: DirectSecp256k1HdWallet, client: SigningCosmWasmClient) {
	const initMsg = {
		name: 'Dorium Social Business Token',
		symbol: 'SOBZ',
		decimals: 2,
		initial_balances: [
			{ address: 'wasm1ryuawewrukex42yh2kpydtpdh90ex096kaajek', amount: '10000000000' }, // whatever, we can mint more later?
		],
		mint: {
			minter: 'wasm1ryuawewrukex42yh2kpydtpdh90ex096kaajek',
		},
	};

	const instanceData = await client.instantiate(account.address, contractData.codeId, initMsg, "instantiating the DORCP contract");
	return instanceData
}

async function instantiateDoriumCommunityProposal(id: string, description: string, cw20Address1: string, contractData: UploadResult, account: AccountData, wallet: DirectSecp256k1HdWallet, client: SigningCosmWasmClient): Promise<[InstantiateResult, ExecuteResult]> {
	const instantiateData = await client.instantiate(
		account.address,
		contractData.codeId,
		{},
		'instantiate() of the Rust smart contract'
	);
	const contractAddress = instantiateData.contractAddress;

	const createMsg = {
		create:{
		description: description,
		id: id,
		proposer: account.address,
		source: account.address,
		validators: [account.address],
		cw20_whitelist: [cw20Address1]
	}}

	const funds = Coin.fromJSON({denom: "ucosm", amount: "1"})
	const createData = await client.execute(account.address, contractAddress, createMsg, "execute_create() of the Rust smart contract", [funds])
	return [instantiateData, createData]
}

async function queryProposalState(id: string, dorcpContractAddress: string, client: SigningCosmWasmClient) {
	const dorcpState = await client.queryContractSmart(dorcpContractAddress, {details: {id: id}})
	return dorcpState
}

async function sendToProposal(id: string, cw20Address: string, dorcpContractAddress: string, amount: string, from: string, client: SigningCosmWasmClient) {
	const topup = {top_up: {id: id}}
	const topupBin = toBase64(toUtf8(JSON.stringify(topup)))
	const transfer = await client.execute(from, cw20Address, {send: {contract: dorcpContractAddress, amount: amount, msg: topupBin}});
	return transfer
}

export async function deploy() {
	try {
		const account = await getWalletAccount();
		const wallet = await getWalletData();
		const client = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, wallet, options);

		var con: any = readContractsJson()

		const inst_cw20_value = await instantiateValueToken(con.contracts.cw20, account, wallet, client);
		console.log("CW20 (Value) Instantiated", inst_cw20_value);
		const inst_cw20_sobz = await instantiateSobzToken(con.contracts.cw20, account, wallet, client);
		console.log("CW20 (Sobz) Instantiated", inst_cw20_sobz);
		const inst_dorcp = await instantiateDoriumCommunityProposal("test-dorcp", "this is just a test Proposal", inst_cw20_value.contractAddress, con.contracts.dorcp, account, wallet, client);
		console.log("DORCP Instantiated", inst_dorcp);
		const output = {
			"valuetoken": inst_cw20_value.contractAddress,
			"sobztoken": inst_cw20_sobz.contractAddress,
			"dorcp": inst_dorcp[0].contractAddress,
		}
		con.deployed_contracts = output;
		writeContractsJson(con)
	} catch (e) {
		throw e;
	}
}

export async function scratchpad() {
	const account = await getWalletAccount();
	const wallet = await getWalletData();
	const client = await SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, wallet, options);

	var c = readContractsJson()
	const token = CW20(client).use(c.deployed_contracts.valuetoken)
	const result = await token.balance(account.address)
	console.log("Master account", account.address, "balance in CW20", result)

	// Sending CW20 to DORCP contract instance
	const transfer = await sendToProposal('test-dorcp', c.deployed_contracts.valuetoken, c.deployed_contracts.dorcp, "1000", account.address, client)
	console.dir(transfer, {depth: null})

	// Querying DORCP contract state
	var dorcpState = await queryProposalState('test-dorcp', c.deployed_contracts.dorcp, client)
	console.dir(dorcpState, {depth: null})

	const result2 = await token.balance(account.address)
	console.log("Master account", account.address, "balance in CW20", result2)
}