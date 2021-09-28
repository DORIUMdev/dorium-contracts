import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { accountFromAny, Coin } from "@cosmjs/stargate";
import { toBase64, toUtf8 } from '@cosmjs/encoding';

interface DORCPInstance {
    readonly contractAddress: string

    // actions
    create: (txSigner: string, id: string, description:string, proposer: string, validators: string[], cw20_whitelist: string[], funds: Coin) => Promise<string>
    topup: (txSigner: string, cw20Address: string, id: string, amountCW20: string) => Promise<string>
    status: (id: string) => Promise<any>
    approve: (txSigner: string, id: string) => Promise<string>
    refund: (txSigner: string, id: string) => Promise<string>
  }

interface DORCPContract {
    use: (contractAddress: string) => DORCPInstance
}

export const DORCP = (client: SigningCosmWasmClient): DORCPContract => {
    const use = (contractAddress: string): DORCPInstance => {
        const create = async (senderAddress: string, id: string, description: string, proposer: string, validators: string[], cw20_whitelist: string[], funds: Coin): Promise<string> => {
            const createMsg = {
                create: {
                    "id": id,
                    "description": description,
                    "validators": validators,
                    "proposer": proposer,
                    "source": "unused",
                    "cw20_whitelist": cw20_whitelist,
                }
            }
            const result = await client.execute(senderAddress, contractAddress, createMsg, "DORCP.create()", [funds])
            return result.transactionHash
        }

        const topup = async (senderAddress: string, cw20Address: string, id: string, amountCW20: string): Promise<string> => {
            // does not actually call the DORCP Escrow contract - this calls the
            // CW20 contract to send CW20 tokens to the Escrow contract, with a
            // message to the Escrow contract that it is for a particular escrow
            // id.
            const topup = {top_up: {id: id}}
            const topupBin = toBase64(toUtf8(JSON.stringify(topup)))
            const result = await client.execute(senderAddress, cw20Address, {send: {contract: contractAddress, amount: amountCW20, msg: topupBin}});
            return result.transactionHash;
        }

        const status = async(id: string): Promise<any> => {
            const result = await client.queryContractSmart(contractAddress, {details: {id: id}})
            return result
        }

        const approve = async (senderAddress: string, id: string): Promise<string> => {
            const result = await client.execute(senderAddress, contractAddress, {approve: {"id": id}});
            return result.transactionHash;
        }
        const refund = async (senderAddress: string, id: string): Promise <string> => {
            const result = await client.execute(senderAddress, contractAddress, {refund: {"id": id}});
            return result.transactionHash
        }
        return {
            contractAddress,
            create,
            topup,
            status,
            approve,
            refund,
        };
    }
    return {use};
}