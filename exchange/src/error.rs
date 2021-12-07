use cosmwasm_std::{Addr, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: info.sender {info_sender:?} is NOT state.value_token_address {state_value_token_address:?}")]
    UnauthorizedValueToken {
        info_sender: Addr,
        state_value_token_address: Addr,
    },

    #[error("Unauthorized Set Token: {sender:?} tried to modify the CW20 Token Addresses but {owner:?} owns the exchange contract")]
    UnauthorizedSetToken { owner: Addr, sender: Addr },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
