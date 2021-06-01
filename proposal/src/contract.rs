use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse};
use crate::state::{State, STATE};
use crate::{error::ContractError, state::Status};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        admin: info.sender,
        proposer: msg.proposer,
        budget: msg.budget,
        validators: msg.validators,
        status: crate::state::Status::Opened {},
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetStatus { s } => try_set_status(deps, info, s),
        // only DORIUM can add/remove validators
        ExecuteMsg::AddValidator { addr } => try_add_validator(deps, info, addr),
        ExecuteMsg::RmValidator { addr } => try_rm_validator(deps, info, addr),
    }
}

pub fn try_set_status(
    deps: DepsMut,
    info: MessageInfo,
    status: Status,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        // only validators can update the state of a Proposal
        if !state.validators.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }
        state.status = status;
        Ok(state)
    })?;

    Ok(Response::default())
}

pub fn try_add_validator(
    deps: DepsMut,
    info: MessageInfo,
    validator: Addr,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        // only DORIUM can add/remove validators to a Proposal
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }

        // check that the address to be added isn't already in there
        if state.validators.contains(&validator) {
            return Err(ContractError::Std(StdError::generic_err(
                "Validator is already registered",
            )));
        }

        state.validators.push(validator);
        Ok(state)
    })?;
    Ok(Response::default())
}

pub fn try_rm_validator(
    deps: DepsMut,
    info: MessageInfo,
    validator: Addr,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        // only DORIUM can add/remove validators to a Proposal
        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }

        // check that the address to be added isn't already in there
        match state.validators.iter().position(|v| *v == validator) {
            Some(i) => {
                state.validators.remove(i);
            }
            None => {
                return Err(
                    StdError::generic_err("Validator was not registered as a validator").into(),
                )
            }
        }
        Ok(state)
    })?;
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_binary(&query_state(deps)?),
    }
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse { state: state })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coins, from_binary, Uint128};
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Coin,
    };

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let vs = vec![Addr::unchecked("validator1"), Addr::unchecked("validator2")];
        let msg = InstantiateMsg {
            proposer: Addr::unchecked("proposer"),
            budget: Coin {
                amount: Uint128(1000),
                denom: "BTC".into(),
            },
            validators: vs,
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let s: StateResponse = from_binary(&res).unwrap();
        let state: State = s.state;
        println!("{:?}", state);
        let state_reference = State {
            admin: Addr::unchecked("creator"),
            proposer: Addr::unchecked("proposer"),
            budget: Coin {
                denom: "BTC".into(),
                amount: Uint128(1000),
            },
            validators: vec![Addr::unchecked("validator1"), Addr::unchecked("validator2")],
            status: Status::Opened {},
        };
        assert_eq!(state, state_reference);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
