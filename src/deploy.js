"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.main = void 0;
var dotenv_1 = require("dotenv");
var fs = require("fs");
var proto_signing_1 = require("@cosmjs/proto-signing");
var cosmwasm_stargate_1 = require("@cosmjs/cosmwasm-stargate");
var cw20_base_helpers_1 = require("./cw20-base-helpers");
dotenv_1.config();
var _a = process.env, ERC20_CONTRACT = _a.ERC20_CONTRACT, ESCROW_CONTRACT = _a.ESCROW_CONTRACT, DORIUM_PROPOSAL_CONTRACT = _a.DORIUM_PROPOSAL_CONTRACT, MNEMONIC_MAIN = _a.MNEMONIC_MAIN, RPC_ENDPOINT = _a.RPC_ENDPOINT;
var options = { prefix: "wasm" };
var ERC20Contract = fs.readFileSync(ERC20_CONTRACT);
var ProposalContract = fs.readFileSync(DORIUM_PROPOSAL_CONTRACT);
function getWalletData() {
    return __awaiter(this, void 0, void 0, function () {
        var wallet_main;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, proto_signing_1.DirectSecp256k1HdWallet.fromMnemonic(MNEMONIC_MAIN, options)];
                case 1:
                    wallet_main = _a.sent();
                    return [2 /*return*/, wallet_main];
            }
        });
    });
}
function getWalletAccount() {
    return __awaiter(this, void 0, void 0, function () {
        var walletData, mainAccount;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, getWalletData()];
                case 1:
                    walletData = _a.sent();
                    return [4 /*yield*/, walletData.getAccounts()];
                case 2:
                    mainAccount = (_a.sent())[0];
                    return [2 /*return*/, mainAccount];
            }
        });
    });
}
function instantiateCW20() {
    return __awaiter(this, void 0, void 0, function () {
        var account, wallet, client, initMsg, contractData, instantiateData, cw20;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, getWalletAccount()];
                case 1:
                    account = _a.sent();
                    return [4 /*yield*/, getWalletData()];
                case 2:
                    wallet = _a.sent();
                    return [4 /*yield*/, cosmwasm_stargate_1.SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, wallet, options)];
                case 3:
                    client = _a.sent();
                    initMsg = {
                        name: "Dorium Value Token",
                        symbol: "TREE",
                        decimals: 2,
                        initial_balances: [
                            { address: "wasm1ryuawewrukex42yh2kpydtpdh90ex096kaajek", amount: "3040000000000" }, // number of trees in the world according to Google
                        ],
                        mint: {
                            minter: "wasm1ryuawewrukex42yh2kpydtpdh90ex096kaajek"
                        }
                    };
                    return [4 /*yield*/, client.upload(account.address, ERC20Contract)];
                case 4:
                    contractData = _a.sent();
                    return [4 /*yield*/, client.instantiate(account.address, contractData.codeId, initMsg, "creating the cw20 token")];
                case 5:
                    instantiateData = _a.sent();
                    console.log('CW20 Value Token Uploaded Contract', contractData);
                    console.log('CW20 Value Token Instantiated Contract', instantiateData);
                    cw20 = cw20_base_helpers_1.CW20(client).use(instantiateData.contractAddress);
                    return [2 /*return*/];
            }
        });
    });
}
function uploadDoriumCommunityProposal() {
    return __awaiter(this, void 0, void 0, function () {
        var account, wallet, client, contractData;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, getWalletAccount()];
                case 1:
                    account = _a.sent();
                    return [4 /*yield*/, getWalletData()];
                case 2:
                    wallet = _a.sent();
                    return [4 /*yield*/, cosmwasm_stargate_1.SigningCosmWasmClient.connectWithSigner(RPC_ENDPOINT, wallet, options)];
                case 3:
                    client = _a.sent();
                    return [4 /*yield*/, client.upload(account.address, ProposalContract)];
                case 4:
                    contractData = _a.sent();
                    console.log('DORium Community Proposal', contractData);
                    return [2 /*return*/];
            }
        });
    });
}
function main() {
    return __awaiter(this, void 0, void 0, function () {
        var e_1;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 3, , 4]);
                    return [4 /*yield*/, instantiateCW20()];
                case 1:
                    _a.sent();
                    return [4 /*yield*/, uploadDoriumCommunityProposal()];
                case 2:
                    _a.sent();
                    return [3 /*break*/, 4];
                case 3:
                    e_1 = _a.sent();
                    throw e_1;
                case 4: return [2 /*return*/];
            }
        });
    });
}
exports.main = main;
