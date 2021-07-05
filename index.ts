import { main, uploadContracts2} from "./src/deploy";

if(process.argv[2] == "upload") {
    uploadContracts2()
} else if(process.argv[2] == "deploy") {
    main().then(r => r);
}