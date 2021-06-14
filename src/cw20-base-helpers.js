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
exports.CW20 = void 0;
var CW20 = function (client) {
    var use = function (contractAddress) {
        var balance = function (account) { return __awaiter(void 0, void 0, void 0, function () {
            var address, result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        address = account || client.senderAddress;
                        return [4 /*yield*/, client.queryContractSmart(contractAddress, { balance: { address: address } })];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, result.balance];
                }
            });
        }); };
        var allowance = function (owner, spender) { return __awaiter(void 0, void 0, void 0, function () {
            return __generator(this, function (_a) {
                return [2 /*return*/, client.queryContractSmart(contractAddress, { allowance: { owner: owner, spender: spender } })];
            });
        }); };
        var allAllowances = function (owner, startAfter, limit) { return __awaiter(void 0, void 0, void 0, function () {
            return __generator(this, function (_a) {
                return [2 /*return*/, client.queryContractSmart(contractAddress, { all_allowances: { owner: owner, start_after: startAfter, limit: limit } })];
            });
        }); };
        var allAccounts = function (startAfter, limit) { return __awaiter(void 0, void 0, void 0, function () {
            var accounts;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, client.queryContractSmart(contractAddress, { all_accounts: { start_after: startAfter, limit: limit } })];
                    case 1:
                        accounts = _a.sent();
                        return [2 /*return*/, accounts.accounts];
                }
            });
        }); };
        var tokenInfo = function () { return __awaiter(void 0, void 0, void 0, function () {
            return __generator(this, function (_a) {
                return [2 /*return*/, client.queryContractSmart(contractAddress, { token_info: {} })];
            });
        }); };
        var minter = function () { return __awaiter(void 0, void 0, void 0, function () {
            return __generator(this, function (_a) {
                return [2 /*return*/, client.queryContractSmart(contractAddress, { minter: {} })];
            });
        }); };
        // mints tokens, returns transactionHash
        var mint = function (recipient, amount) { return __awaiter(void 0, void 0, void 0, function () {
            var result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, client.execute(contractAddress, { mint: { recipient: recipient, amount: amount } })];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, result.transactionHash];
                }
            });
        }); };
        // transfers tokens, returns transactionHash
        var transfer = function (recipient, amount) { return __awaiter(void 0, void 0, void 0, function () {
            var result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, client.execute(contractAddress, { transfer: { recipient: recipient, amount: amount } })];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, result.transactionHash];
                }
            });
        }); };
        // burns tokens, returns transactionHash
        var burn = function (amount) { return __awaiter(void 0, void 0, void 0, function () {
            var result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, client.execute(contractAddress, { burn: { amount: amount } })];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, result.transactionHash];
                }
            });
        }); };
        var increaseAllowance = function (spender, amount) { return __awaiter(void 0, void 0, void 0, function () {
            var result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, client.execute(contractAddress, { increase_allowance: { spender: spender, amount: amount } })];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, result.transactionHash];
                }
            });
        }); };
        var decreaseAllowance = function (spender, amount) { return __awaiter(void 0, void 0, void 0, function () {
            var result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, client.execute(contractAddress, { decrease_allowance: { spender: spender, amount: amount } })];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, result.transactionHash];
                }
            });
        }); };
        var transferFrom = function (owner, recipient, amount) { return __awaiter(void 0, void 0, void 0, function () {
            var result;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0: return [4 /*yield*/, client.execute(contractAddress, { transfer_from: { owner: owner, recipient: recipient, amount: amount } })];
                    case 1:
                        result = _a.sent();
                        return [2 /*return*/, result.transactionHash];
                }
            });
        }); };
        return {
            contractAddress: contractAddress,
            balance: balance,
            allowance: allowance,
            allAllowances: allAllowances,
            allAccounts: allAccounts,
            tokenInfo: tokenInfo,
            minter: minter,
            mint: mint,
            transfer: transfer,
            burn: burn,
            increaseAllowance: increaseAllowance,
            decreaseAllowance: decreaseAllowance,
            transferFrom: transferFrom,
        };
    };
    var downloadWasm = function (url) { return __awaiter(void 0, void 0, void 0, function () {
        var r;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, axios.get(url, { responseType: 'arraybuffer' })];
                case 1:
                    r = _a.sent();
                    if (r.status !== 200) {
                        throw new Error("Download error: " + r.status);
                    }
                    return [2 /*return*/, r.data];
            }
        });
    }); };
    var upload = function () { return __awaiter(void 0, void 0, void 0, function () {
        var meta, sourceUrl, wasm, result;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    meta = {
                        source: "https://github.com/CosmWasm/cosmwasm-plus/tree/v0.4.0/contracts/cw20-base",
                        builder: "cosmwasm/workspace-optimizer:0.10.7"
                    };
                    sourceUrl = "https://github.com/CosmWasm/cosmwasm-plus/releases/download/v0.4.0/cw20_base.wasm";
                    return [4 /*yield*/, downloadWasm(sourceUrl)];
                case 1:
                    wasm = _a.sent();
                    return [4 /*yield*/, client.upload(wasm, meta)];
                case 2:
                    result = _a.sent();
                    return [2 /*return*/, result.codeId];
            }
        });
    }); };
    var instantiate = function (codeId, initMsg, label, admin) { return __awaiter(void 0, void 0, void 0, function () {
        var result;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, client.instantiate(codeId, initMsg, label, { memo: "Init " + label, admin: admin })];
                case 1:
                    result = _a.sent();
                    return [2 /*return*/, use(result.contractAddress)];
            }
        });
    }); };
    return { upload: upload, instantiate: instantiate, use: use };
};
exports.CW20 = CW20;
