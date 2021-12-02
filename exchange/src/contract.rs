#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20::{Cw20Contract, Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::error::ContractError;
use crate::msg::{ExchangedResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:exchange";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        exchanged: Uint128::zero(),
        value_token_address: Addr::from(info.sender.clone()),
        sobz_token_address: Addr::from(info.sender),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("exchanged", "0")
        .add_attribute("value_token_address", state.value_token_address)
        .add_attribute("sobz_token_address", state.sobz_token_address))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => try_exchange(deps, info, msg),
        ExecuteMsg::SetTokens {
            value_token_address,
            sobz_token_address,
        } => set_tokens(deps, info, value_token_address, sobz_token_address),
    }
}

pub fn try_exchange(
    deps: DepsMut,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // did the Cw20ReceiveMsg come from the Value Token Contract? If so, continue.
    let state = STATE.load(deps.storage)?;
    if info.sender != state.value_token_address {
        return Err(ContractError::UnauthorizedValueToken {
            info_sender: info.sender,
            state_value_token_address: state.value_token_address,
        });
    }

    // send message to Cw20 Value Token to burn
    let value_token = Cw20Contract(state.value_token_address);
    let value_token_msg = value_token.call(Cw20ExecuteMsg::Burn { amount: msg.amount })?;

    // send message to Cw20 Sobz Token to mint
    let sobz_token = Cw20Contract(state.sobz_token_address);
    // msg: Cw20ReceiveMsg.sender is the keypair that actually initiated the
    // whole transaction; info.sender is the Cw20 contract that emitted the
    // Cw20ReceiveMsg to us
    let sobz_token_msg = sobz_token.call(Cw20ExecuteMsg::Mint {
        recipient: msg.sender.clone(),
        amount: msg.amount,
    })?;

    // update # of valuetokens exchanged counter
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.exchanged = state.exchanged + msg.amount;
        Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("method", "try_exchange")
        .add_attribute("account", msg.sender)
        .add_attribute("exchange", stringify!(msg.amount))
        .add_message(value_token_msg)
        .add_message(sobz_token_msg))
}

pub fn set_tokens(
    deps: DepsMut,
    info: MessageInfo,
    value_token_address: Addr,
    sobz_token_address: Addr,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    state.value_token_address = value_token_address.clone();
    state.sobz_token_address = sobz_token_address.clone();

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "set_tokens")
        .add_attribute("value_token", value_token_address)
        .add_attribute("sobz_token", sobz_token_address))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetExchanged {} => to_binary(&query_exchanged(deps)?),
    }
}

fn query_exchanged(deps: Deps) -> StdResult<ExchangedResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(ExchangedResponse {
        exchanged: state.exchanged,
    })
}

#[cfg(test)]
mod tests {
    use crate::msg::ReceiveMsg;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, CosmosMsg, WasmMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            value_token_address: Addr::unchecked("TREE"),
            sobz_token_address: Addr::unchecked("SOBZ"),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetExchanged {}).unwrap();
        let value: ExchangedResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::zero(), value.exchanged);
    }

    #[test]
    fn exchange() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {
            value_token_address: Addr::unchecked("TREE"),
            sobz_token_address: Addr::unchecked("SOBZ"),
        };
        let info = mock_info("Dorium", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("TREE", &coins(2, "token"));
        let receive_msg = Cw20ReceiveMsg {
            sender: String::from("some user"),
            amount: Uint128::new(18),
            msg: to_binary(&ReceiveMsg::Send {}).unwrap(),
        };
        let execute_msg = ExecuteMsg::Receive(receive_msg);
        let res = execute(deps.as_mut(), mock_env(), info, execute_msg).unwrap();

        // check that exchange contract told TREE and SOBZ CW20 contracts to Mint and Burn respectively
        println!("{:?}", res.messages);
        let value_token_msg = res.messages[0].clone().msg;
        assert_eq!(
            value_token_msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: String::from("TREE"),
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: Uint128::new(18)
                })
                .unwrap(),
                funds: vec![],
            })
        );
        let sobz_token_msg = res.messages[1].clone().msg;
        assert_eq!(
            sobz_token_msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: String::from("SOBZ"),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: String::from("some user"),
                    amount: Uint128::new(18)
                })
                .unwrap(),
                funds: vec![],
            })
        );

        // exchange contract's counter should've increased by <amount>
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetExchanged {}).unwrap();
        let value: ExchangedResponse = from_binary(&res).unwrap();
        assert_eq!(Uint128::new(18), value.exchanged);
    }
}
