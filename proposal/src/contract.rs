use crate::state::{all_escrow_details, all_escrow_ids, Escrow, GenericBalance, ESCROWS};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, Response, StdError, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Balance, Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::error::ContractError;
use crate::msg::{
    CreateMsg, DetailsResponse, ExecuteMsg, InstantiateMsg, ListDetailedResponse, ListResponse,
    QueryMsg, ReceiveMsg,
};
use crate::state::Status;

// version info for migration info
const CONTRACT_NAME: &str = "dorium-community-proposal";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // no setup
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Create(msg) => {
            execute_create(deps, msg, Balance::from(info.funds), &info.sender)
        }
        ExecuteMsg::Approve { id } => execute_approve(deps, env, info, id),
        ExecuteMsg::TopUp { id } => execute_top_up(deps, id, Balance::from(info.funds)),
        ExecuteMsg::Refund { id } => execute_refund(deps, env, info, id),
        ExecuteMsg::Receive(msg) => execute_receive(deps, info, msg),
    }
}

pub fn execute_receive(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let balance = Balance::Cw20(Cw20CoinVerified {
        address: info.sender,
        amount: wrapper.amount,
    });
    let api = deps.api;
    match msg {
        ReceiveMsg::Create(msg) => {
            execute_create(deps, msg, balance, &api.addr_validate(&wrapper.sender)?)
        }
        ReceiveMsg::TopUp { id } => execute_top_up(deps, id, balance),
    }
}

pub fn execute_create(
    deps: DepsMut,
    msg: CreateMsg,
    balance: Balance,
    sender: &Addr,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }

    let mut cw20_whitelist = msg.addr_whitelist(deps.api)?;

    let escrow_balance = match balance {
        Balance::Native(balance) => GenericBalance {
            native: balance.0,
            cw20: vec![],
        },
        Balance::Cw20(token) => {
            // make sure the token sent is on the whitelist by default
            if !cw20_whitelist.iter().any(|t| t == &token.address) {
                cw20_whitelist.push(token.address.clone())
            }
            GenericBalance {
                native: vec![],
                cw20: vec![token],
            }
        }
    };

    let mut validators: Vec<Addr> = vec![];
    for addr in msg.validators {
        validators.push(deps.api.addr_validate(&addr)?)
    }

    let escrow = Escrow {
        id: msg.id.clone(),
        url: msg.url.clone(),
        description: msg.description,
        validators: validators,
        proposer: deps.api.addr_validate(&msg.proposer)?,
        source: sender.clone(),
        balance: escrow_balance,
        cw20_whitelist,
        status: Status::Opened {},
    };

    // try to store it, fail if the id was already in use
    ESCROWS.update(deps.storage, &msg.id, |existing| match existing {
        None => Ok(escrow),
        Some(_) => Err(ContractError::AlreadyInUse {}),
    })?;

    let res = Response {
        attributes: vec![attr("action", "create"), attr("id", msg.id)],
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_top_up(
    deps: DepsMut,
    id: String,
    balance: Balance,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }
    // this fails is no escrow there
    let mut escrow = ESCROWS.load(deps.storage, &id)?;

    // If status is Completed or Canceled, don't let people send tokens to this escrow anymore!
    if escrow.locked() {
        return Err(ContractError::Locked {});
    }

    if let Balance::Cw20(token) = &balance {
        // ensure the token is on the whitelist
        if !escrow.cw20_whitelist.iter().any(|t| t == &token.address) {
            return Err(ContractError::NotInWhitelist {});
        }
    };

    escrow.balance.add_tokens(balance);

    // and save
    ESCROWS.save(deps.storage, &id, &escrow)?;

    let res = Response {
        attributes: vec![attr("action", "top_up"), attr("id", id)],
        ..Response::default()
    };
    Ok(res)
}

pub fn execute_approve(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response, ContractError> {
    // this fails is no escrow there
    let mut escrow = ESCROWS.load(deps.storage, &id)?;

    if !escrow.validators.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    } else if escrow.locked() {
        return Err(ContractError::Locked {});
    }
    {
        escrow.status = Status::Completed {};

        // send all tokens out
        let messages = send_tokens(&escrow.proposer, &escrow.balance)?;

        // save the updated status field
        ESCROWS.save(deps.storage, &id, &escrow)?;

        let attributes = vec![
            attr("action", "approve"),
            attr("id", id),
            attr("to", escrow.proposer),
        ];
        Ok(Response {
            submessages: vec![],
            messages,
            attributes,
            data: None,
        })
    }
}

pub fn execute_refund(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response, ContractError> {
    // this fails is no escrow there
    let mut escrow = ESCROWS.load(deps.storage, &id)?;

    // only a validator can decide to refund the escrowed funds (to DORIUM)
    if !escrow.validators.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    } else if escrow.locked() {
        return Err(ContractError::Locked {});
    } else {
        escrow.status = Status::Canceled {};

        // send all tokens out
        let messages = refund_or_burn_tokens(&escrow.source, &escrow.balance)?;

        // save the updated status field
        ESCROWS.save(deps.storage, &id, &escrow)?;

        let attributes = vec![
            attr("action", "refund"),
            attr("id", id),
            attr("to", escrow.source),
        ];
        Ok(Response {
            submessages: vec![],
            messages,
            attributes,
            data: None,
        })
    }
}

fn send_tokens(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<CosmosMsg>> {
    let native_balance = &balance.native;
    let mut msgs: Vec<CosmosMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        vec![BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.to_vec(),
        }
        .into()]
    };

    let cw20_balance = &balance.cw20;
    let cw20_msgs: StdResult<Vec<_>> = cw20_balance
        .iter()
        .map(|c| {
            let msg = Cw20ExecuteMsg::Transfer {
                recipient: to.into(),
                amount: c.amount,
            };
            let exec = WasmMsg::Execute {
                contract_addr: c.address.to_string(),
                msg: to_binary(&msg)?,
                send: vec![],
            };
            Ok(exec.into())
        })
        .collect();
    msgs.append(&mut cw20_msgs?);
    Ok(msgs)
}

fn refund_or_burn_tokens(to: &Addr, balance: &GenericBalance) -> StdResult<Vec<CosmosMsg>> {
    let native_balance = &balance.native;
    let mut msgs: Vec<CosmosMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        vec![BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.to_vec(),
        }
        .into()]
    };

    let cw20_balance = &balance.cw20;
    let cw20_msgs: StdResult<Vec<_>> = cw20_balance
        .iter()
        .map(|c| {
            let msg = Cw20ExecuteMsg::Burn { amount: c.amount };
            let exec = WasmMsg::Execute {
                contract_addr: c.address.to_string(),
                msg: to_binary(&msg)?,
                send: vec![],
            };
            Ok(exec.into())
        })
        .collect();
    msgs.append(&mut cw20_msgs?);
    Ok(msgs)
}

impl DetailsResponse {
    fn from_escrow(escrow: &Escrow) -> StdResult<DetailsResponse> {
        let cw20_whitelist = escrow.human_whitelist();
        let validators_str = escrow.human_validators();
        // transform tokens
        let native_balance = escrow.balance.native.clone();

        let cw20_balance: StdResult<Vec<_>> = escrow
            .balance
            .cw20
            .clone()
            .into_iter()
            .map(|token| {
                Ok(Cw20Coin {
                    address: token.address.into(),
                    amount: token.amount,
                })
            })
            .collect();

        Ok(DetailsResponse {
            id: escrow.id.clone(),
            url: escrow.url.clone(),
            description: escrow.description.clone(),
            validators: validators_str,
            proposer: escrow.proposer.to_string(),
            source: escrow.source.to_string(),
            native_balance: native_balance.to_vec(),
            cw20_balance: cw20_balance?,
            cw20_whitelist: cw20_whitelist,
            status: escrow.status.clone(),
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::List {} => to_binary(&query_list(deps)?),
        QueryMsg::ListDetailed {} => to_binary(&query_list_detailed(deps)?),
        QueryMsg::Details { id } => to_binary(&query_details(deps, id)?),
    }
}

fn query_details(deps: Deps, id: String) -> StdResult<DetailsResponse> {
    let escrow = ESCROWS.load(deps.storage, &id)?;
    Ok(DetailsResponse::from_escrow(&escrow)?)
}

fn query_list(deps: Deps) -> StdResult<ListResponse> {
    Ok(ListResponse {
        escrows: all_escrow_ids(deps.storage)?,
    })
}

fn query_list_detailed(deps: Deps) -> StdResult<ListDetailedResponse> {
    let ids = all_escrow_ids(deps.storage)?;
    let escrows = all_escrow_details(deps.storage, ids)?;
    let drs: StdResult<Vec<DetailsResponse>> = escrows
        .iter()
        .map(|e: &Escrow| DetailsResponse::from_escrow(e))
        .collect();
    Ok(ListDetailedResponse { escrows: drs? })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, Coin, CosmosMsg, StdError, Uint128};

    use crate::msg::ExecuteMsg::TopUp;

    use super::*;
    fn mock_topup_cw20_message(id: &String) -> StdResult<ExecuteMsg> {
        let base = TopUp { id: id.to_string() };
        let top_up = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: String::from("random"),
            amount: Uint128(7890),
            msg: to_binary(&base).unwrap(),
        });
        return Ok(top_up);
    }

    fn quick_create_msg_cw20() -> (CreateMsg, ExecuteMsg, MessageInfo) {
        let create = CreateMsg {
            id: "foobar".to_string(),
            url: "https://darmstadt.dorium.apeunit.com".to_string(),
            description: String::from("foo to a bar"),
            validators: vec![String::from("validator1"), String::from("validator2")],
            proposer: String::from("recd"),
            source: String::from("dorium"),
            cw20_whitelist: Some(vec![String::from("other-token")]),
        };
        let receive = Cw20ReceiveMsg {
            sender: String::from("dorium"),
            amount: Uint128(100),
            msg: to_binary(&ExecuteMsg::Create(create.clone())).unwrap(),
        };
        let token_contract = String::from("my-cw20-token");
        let info = mock_info(&token_contract, &[]);
        let msg = ExecuteMsg::Receive(receive.clone());
        return (create, msg, info);
    }
    #[test]
    fn approve_proposal_native_token() {
        let mut deps = mock_dependencies(&[]);

        // instantiate an empty contract
        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(&String::from("anyone"), &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // create an escrow
        let create = CreateMsg {
            id: "foobar".to_string(),
            url: "https://darmstadt.dorium.apeunit.com".to_string(),
            description: String::from("foo of a bar of a escrow"),
            validators: vec![String::from("validator1"), String::from("validator2")],
            proposer: String::from("recd"),
            source: String::from("dorium"),
            cw20_whitelist: None,
        };
        let sender = String::from("dorium");
        let balance = coins(100, "tokens");
        let info = mock_info(&sender, &balance);
        let msg = ExecuteMsg::Create(create.clone());
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(attr("action", "create"), res.attributes[0]);

        // ensure the details is what we expect
        let details = query_details(deps.as_ref(), "foobar".to_string()).unwrap();
        assert_eq!(
            details,
            DetailsResponse {
                id: "foobar".to_string(),
                url: "https://darmstadt.dorium.apeunit.com".to_string(),
                description: String::from("foo of a bar of a escrow"),
                validators: vec![String::from("validator1"), String::from("validator2")],
                proposer: String::from("recd"),
                source: String::from("dorium"),
                native_balance: balance.clone(),
                cw20_balance: vec![],
                cw20_whitelist: vec![],
                status: Status::Opened {},
            }
        );

        // approve it
        let id = create.id.clone();
        let info = mock_info(&create.validators[0], &[]);
        let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Approve { id }).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(attr("action", "approve"), res.attributes[0]);
        assert_eq!(
            res.messages[0],
            CosmosMsg::Bank(BankMsg::Send {
                to_address: create.proposer,
                amount: balance,
            })
        );
    }

    #[test]
    fn approve_proposal_cw20_token() {
        let mut deps = mock_dependencies(&[]);

        // instantiate an empty contract
        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(&String::from("anyone"), &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // create an escrow
        let create = CreateMsg {
            id: "foobar".to_string(),
            url: "https://darmstadt.dorium.apeunit.com".to_string(),
            description: String::from("foo to a bar"),
            validators: vec![String::from("validator1"), String::from("validator2")],
            proposer: String::from("recd"),
            source: String::from("dorium"),
            cw20_whitelist: Some(vec![String::from("other-token")]),
        };
        let receive = Cw20ReceiveMsg {
            sender: String::from("dorium"),
            amount: Uint128(100),
            msg: to_binary(&ExecuteMsg::Create(create.clone())).unwrap(),
        };
        let token_contract = String::from("my-cw20-token");
        let info = mock_info(&token_contract, &[]);
        let msg = ExecuteMsg::Receive(receive.clone());
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(attr("action", "create"), res.attributes[0]);

        // ensure the whitelist is what we expect
        let details = query_details(deps.as_ref(), "foobar".to_string()).unwrap();
        assert_eq!(
            details,
            DetailsResponse {
                id: "foobar".to_string(),
                url: "https://darmstadt.dorium.apeunit.com".to_string(),
                description: String::from("foo to a bar"),
                validators: vec![String::from("validator1"), String::from("validator2")],
                proposer: String::from("recd"),
                source: String::from("dorium"),
                native_balance: vec![],
                cw20_balance: vec![Cw20Coin {
                    address: String::from("my-cw20-token"),
                    amount: Uint128(100),
                }],
                cw20_whitelist: vec![String::from("other-token"), String::from("my-cw20-token")],
                status: Status::Opened {},
            }
        );

        // approve it
        let info = mock_info(&create.validators[0], &[]);
        let res = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Approve {
                id: create.id.clone(),
            },
        )
        .unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(attr("action", "approve"), res.attributes[0]);
        let send_msg = Cw20ExecuteMsg::Transfer {
            recipient: create.proposer,
            amount: receive.amount,
        };
        assert_eq!(
            res.messages[0],
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token_contract,
                msg: to_binary(&send_msg).unwrap(),
                send: vec![],
            })
        );
    }

    #[test]
    fn reject_proposal_cw20_token() {
        let mut deps = mock_dependencies(&[]);

        // instantiate an empty contract
        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(&String::from("anyone"), &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // create an escrow
        let create = CreateMsg {
            id: "foobar".to_string(),
            url: "https://darmstadt.dorium.apeunit.com".to_string(),
            description: String::from("foo to a bar"),
            validators: vec![String::from("validator1"), String::from("validator2")],
            proposer: String::from("recd"),
            source: String::from("dorium"),
            cw20_whitelist: Some(vec![String::from("other-token")]),
        };
        let receive = Cw20ReceiveMsg {
            sender: String::from("dorium"),
            amount: Uint128(100),
            msg: to_binary(&ExecuteMsg::Create(create.clone())).unwrap(),
        };
        let token_contract = String::from("my-cw20-token");
        let info = mock_info(&token_contract, &[]);
        let msg = ExecuteMsg::Receive(receive.clone());
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(attr("action", "create"), res.attributes[0]);

        // reject it
        let id = create.id.clone();
        let info = mock_info(&create.validators[0], &[]);
        let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Refund { id }).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(attr("action", "refund"), res.attributes[0]);

        // ensure that the escrow contract told the CW20 contract to burn the tokens
        let burn_msg = Cw20ExecuteMsg::Burn {
            amount: Uint128(100),
        };
        assert_eq!(
            res.messages[0],
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: token_contract,
                msg: to_binary(&burn_msg).unwrap(),
                send: vec![],
            })
        );
    }
    #[test]
    fn top_up_mixed_tokens() {
        let mut deps = mock_dependencies(&[]);

        // instantiate an empty contract
        let instantiate_msg = InstantiateMsg {};
        let info = mock_info(&String::from("anyone"), &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // only accept these tokens
        let whitelist = vec![String::from("bar_token"), String::from("foo_token")];

        // create an escrow with 2 native tokens
        let create = CreateMsg {
            id: "foobar".to_string(),
            url: "https://darmstadt.dorium.apeunit.com".to_string(),
            description: String::from("foo to a bar"),
            validators: vec![String::from("validator1"), String::from("validator2")],
            proposer: String::from("recd"),
            source: String::from("dorium"),
            cw20_whitelist: Some(whitelist),
        };
        let sender = String::from("source");
        let balance = vec![coin(100, "fee"), coin(200, "stake")];
        let info = mock_info(&sender, &balance);
        let msg = ExecuteMsg::Create(create.clone());
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(attr("action", "create"), res.attributes[0]);

        // top it up with 2 more native tokens
        let extra_native = vec![coin(250, "random"), coin(300, "stake")];
        let info = mock_info(&sender, &extra_native);
        let top_up = ExecuteMsg::TopUp {
            id: create.id.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, top_up).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(attr("action", "top_up"), res.attributes[0]);

        // top up with one foreign token
        let bar_token = String::from("bar_token");
        let base = TopUp {
            id: create.id.clone(),
        };
        let top_up = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: String::from("random"),
            amount: Uint128(7890),
            msg: to_binary(&base).unwrap(),
        });
        let info = mock_info(&bar_token, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, top_up).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(attr("action", "top_up"), res.attributes[0]);

        // top with a foreign token not on the whitelist
        // top up with one foreign token
        let baz_token = String::from("baz_token");
        let base = TopUp {
            id: create.id.clone(),
        };
        let top_up = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: String::from("random"),
            amount: Uint128(7890),
            msg: to_binary(&base).unwrap(),
        });
        let info = mock_info(&baz_token, &[]);
        let err = execute(deps.as_mut(), mock_env(), info, top_up).unwrap_err();
        assert_eq!(err, ContractError::NotInWhitelist {});

        // top up with second foreign token
        let foo_token = String::from("foo_token");
        let base = TopUp {
            id: create.id.clone(),
        };
        let top_up = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: String::from("random"),
            amount: Uint128(888),
            msg: to_binary(&base).unwrap(),
        });
        let info = mock_info(&foo_token, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, top_up).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(attr("action", "top_up"), res.attributes[0]);

        // approve it
        let id = create.id.clone();
        let info = mock_info(&create.validators[0], &[]);
        let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Approve { id }).unwrap();
        assert_eq!(attr("action", "approve"), res.attributes[0]);
        assert_eq!(3, res.messages.len());

        // first message releases all native coins
        assert_eq!(
            res.messages[0],
            CosmosMsg::Bank(BankMsg::Send {
                to_address: create.proposer.clone(),
                amount: vec![coin(100, "fee"), coin(500, "stake"), coin(250, "random")],
            })
        );

        // second one release bar cw20 token
        let send_msg = Cw20ExecuteMsg::Transfer {
            recipient: create.proposer.clone(),
            amount: Uint128(7890),
        };
        assert_eq!(
            res.messages[1],
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: bar_token,
                msg: to_binary(&send_msg).unwrap(),
                send: vec![],
            })
        );

        // third one release foo cw20 token
        let send_msg = Cw20ExecuteMsg::Transfer {
            recipient: create.proposer,
            amount: Uint128(888),
        };
        assert_eq!(
            res.messages[2],
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: foo_token,
                msg: to_binary(&send_msg).unwrap(),
                send: vec![],
            })
        );
    }

    #[test]
    fn approved_proposal_is_locked() {
        // quickly create a proposal, funding it with cw20 tokens
        let mut deps = mock_dependencies(&[]);
        let (create, msg, info) = quick_create_msg_cw20();
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // approve it
        let id = create.id.clone();
        let info = mock_info(&create.validators[0], &[]);
        execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Approve { id }).unwrap();

        // now that it's approved, topping up with further tokens fails (contract is locked)
        let info = mock_info(&create.validators[0], &[]);
        let top_up = mock_topup_cw20_message(&create.id).unwrap();
        let err = execute(deps.as_mut(), mock_env(), info.clone(), top_up).unwrap_err();
        assert!(matches!(err, ContractError::Locked { .. }));

        // now that it's approved, you can't approve again
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Approve {
                id: create.id.clone(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Locked { .. }));

        // now that it's approved, you can't reject
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Refund {
                id: create.id.clone(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Locked { .. }));
    }

    #[test]
    fn rejected_proposal_is_locked() {
        // quickly create a proposal, funding it with cw20 tokens
        let mut deps = mock_dependencies(&[]);
        let (create, msg, info) = quick_create_msg_cw20();
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // approve it
        let id = create.id.clone();
        let info = mock_info(&create.validators[0], &[]);
        execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Refund { id }).unwrap();

        // now that it's rejected, topping up with further tokens fails (contract is locked)
        let info = mock_info(&create.validators[0], &[]);
        let top_up = mock_topup_cw20_message(&create.id).unwrap();
        let err = execute(deps.as_mut(), mock_env(), info.clone(), top_up).unwrap_err();
        assert!(matches!(err, ContractError::Locked { .. }));

        // now that it's rejected, you can't approve again
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Approve {
                id: create.id.clone(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Locked { .. }));

        // now that it's rejected, you can't reject
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Refund {
                id: create.id.clone(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Locked { .. }));
    }
}
