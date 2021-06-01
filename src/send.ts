import { config } from 'dotenv';
import {
  Secp256k1HdWallet,
  SigningCosmosClient,
  coins,
  coin,
  MsgDelegate,
} from '@cosmjs/launchpad';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';

config();

const { MNEMONIC_VALIDATOR, RPC_ENDPOINT } = process.env;
const options = { prefix: "wasm" };

async function getWalletData() {
	const wallet_validtr = await DirectSecp256k1HdWallet.fromMnemonic(MNEMONIC_VALIDATOR, options);
    return wallet_validtr;
}

export async function send() {
  try {
    const wallet = await Secp256k1HdWallet.fromMnemonic(MNEMONIC_VALIDATOR, options);
    const [{ address }] = await wallet.getAccounts()
    console.log('Address:', address)

    // Ensure the address has some tokens to spend

    // const lcdApi = 'https://…'
    // const client = new SigningCosmosClient(lcdApi, address, wallet)

    // // …

    // const msg: MsgDelegate = {
    //   type: 'cosmos-sdk/MsgDelegate',
    //   value: {
    //     delegator_address: address,
    //     validator_address:
    //       'cosmosvaloper1yfkkk04ve8a0sugj4fe6q6zxuvmvza8r3arurr',
    //     amount: coin(300000, 'ustake'),
    //   },
    // }

    // const fee = {
    //   amount: coins(2000, 'ucosm'),
    //   gas: '180000', // 180k
    // }
    // await client.signAndBroadcast([msg], fee)
  } catch (error) {
      throw error
  }
}
