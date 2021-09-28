import { deploy, uploadContracts2, dorcpLifecycle} from "./src/deploy";
(async () => {
    if(process.argv[2] == "deploy") {
        await uploadContracts2()
        await deploy()
    } else if(process.argv[2] == "custom") {
        await dorcpLifecycle()
    }
})();