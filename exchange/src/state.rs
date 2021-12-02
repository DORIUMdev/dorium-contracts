use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub exchanged: Uint128,
    pub value_token_address: Addr,
    pub sobz_token_address: Addr,
}

pub const STATE: Item<State> = Item::new("state");
