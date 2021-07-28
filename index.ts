import { deploy, uploadContracts2, scratchpad} from "./src/deploy";

void async function main() {
    if(process.argv[2] == "deploy") {
        await uploadContracts2()
        await deploy()
    } else if(process.argv[2] == "custom") {
        await scratchpad()
    }

}