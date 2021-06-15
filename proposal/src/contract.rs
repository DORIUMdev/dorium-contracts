use crate::state::{all_escrow_ids, Escrow, GenericBalance, ESCROWS};
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult,
};
use cw2::set_contract_version;
use cw20::Balance;

use crate::error::ContractError;
use crate::msg::{CreateMsg, ExecuteMsg, InstantiateMsg};
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
        ExecuteMsg::Approve { id } => (),
        ExecuteMsg::TopUp { id } => (),
        ExecuteMsg::Refund { id } => (),
        ExecuteMsg::Receive(msg) => (),
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
        id: msg.id,
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
