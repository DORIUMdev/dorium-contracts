import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

interface DORCPInstance {
    readonly contractAddress: string

    // actions
    create: (txSigner: string, description:string, proposer: string, amount: string) => Promise<string>
    approve: (txSigner: string, id: string) => Promise<string>
    refund: (txSigner: string, id: string) => Promise<string>
  }

interface DORCPContract {
    use: (contractAddress: string) => DORCPInstance
}

export const DORCP = (client: SigningCosmWasmClient): DORCPContract => {
    const use = (contractAddress: string): DORCPInstance => {
        const create = async (senderAddress: string, description: string, proposer: string, amount: string): Promise<string> => {
            const result = await client.execute(senderAddress, contractAddress, {create: {"description": description, "proposer": proposer, "amount": amount}, amount});
            return result.transactionHash
        }
        const approve = async (senderAddress: string, id: string): Promise<string> => {
            const result = await client.execute(senderAddress, contractAddress, {approve: id});
            return result.transactionHash;
        }
        const refund = async (senderAddress: string, id: string): Promise <string> => {
            const result = await client.execute(senderAddress, contractAddress, {refund: id});
            return result.transactionHash
        }
        return {
            contractAddress,
            create,
            approve,
            refund,
        };
    }
    return {use};
}