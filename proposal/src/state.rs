use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Order, StdError, StdResult, Storage};
use cw_storage_plus::Map;

use cw20::{Balance, Cw20CoinVerified};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
}

impl GenericBalance {
    pub fn add_tokens(&mut self, add: Balance) {
        match add {
            Balance::Native(balance) => {
                for token in balance.0 {
                    let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                        if exist.denom == token.denom {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    match index {
                        Some(idx) => self.native[idx].amount += token.amount,
                        None => self.native.push(token),
                    }
                }
            }
            Balance::Cw20(token) => {
                let index = self.cw20.iter().enumerate().find_map(|(i, exist)| {
                    if exist.address == token.address {
                        Some(i)
                    } else {
                        None
                    }
                });
                match index {
                    Some(idx) => self.cw20[idx].amount += token.amount,
                    None => self.cw20.push(token),
                }
            }
        };
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Status {
    Opened {},
    InProgress {},
    Canceled {},
    Completed {},
}
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Escrow {
    /// id is a human-readable name. It is the key in the mapping to store the
    /// Escrow state, so it cannot be changed. 3-20 bytes of utf-8 text
    pub id: String,
    /// URL (there must be a forum post somewhere)
    pub url: String,
    /// more information about this proposal (URL to forum topic?)
    pub description: String,
    /// validators assigned by Dorium can decide to approve or refund the escrow
    pub validators: Vec<Addr>,
    /// if approved, funds go to the proposer
    pub proposer: Addr,
    /// if refunded, funds go to the source (Dorium)
    pub source: Addr,
    /// Balance in Native and Cw20 tokens
    pub balance: GenericBalance,
    /// All possible contracts that we accept tokens from
    pub cw20_whitelist: Vec<Addr>,
    /// status of the proposal (enum: opened, in progress, canceled, completed)
    pub status: Status,
}

impl Escrow {
    pub fn human_whitelist(&self) -> Vec<String> {
        self.cw20_whitelist.iter().map(|a| a.to_string()).collect()
    }

    pub fn human_validators(&self) -> Vec<String> {
        self.validators.iter().map(|a| a.to_string()).collect()
    }
}

pub const ESCROWS: Map<&str, Escrow> = Map::new("escrow");

/// This returns the list of ids for all registered escrows
pub fn all_escrow_ids(storage: &dyn Storage) -> StdResult<Vec<String>> {
    ESCROWS
        .keys(storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).map_err(|_| StdError::invalid_utf8("parsing escrow key")))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::MockStorage;
    use cosmwasm_std::{coin, Uint128};

    #[test]
    fn no_escrow_ids() {
        let storage = MockStorage::new();
        let ids = all_escrow_ids(&storage).unwrap();
        assert_eq!(0, ids.len());
    }

    fn dummy_escrow() -> Escrow {
        Escrow {
            id: "test escrow".to_string(),
            url: "https://darmstadt.dorium.apeunit.com".to_string(),
            description: "more information about the test here".to_string(),
            validators: vec![Addr::unchecked("arb")],
            proposer: Addr::unchecked("proposer"),
            source: Addr::unchecked("source"),
            balance: Default::default(),
            cw20_whitelist: vec![Addr::unchecked("Cw20 Value Token")],
            status: Status::Opened {},
        }
    }

    #[test]
    fn add_tokens_native() {
        let mut tokens = GenericBalance::default();
        tokens.add_tokens(Balance::from(vec![coin(123, "atom"), coin(789, "eth")]));
        tokens.add_tokens(Balance::from(vec![coin(456, "atom"), coin(12, "btc")]));
        assert_eq!(
            tokens.native,
            vec![coin(579, "atom"), coin(789, "eth"), coin(12, "btc")]
        );
    }

    #[test]
    fn add_tokens_cw20() {
        let mut tokens = GenericBalance::default();
        let bar_token = Addr::unchecked("bar_token");
        let foo_token = Addr::unchecked("foo_token");
        tokens.add_tokens(Balance::Cw20(Cw20CoinVerified {
            address: foo_token.clone(),
            amount: Uint128(12345),
        }));
        tokens.add_tokens(Balance::Cw20(Cw20CoinVerified {
            address: bar_token.clone(),
            amount: Uint128(777),
        }));
        tokens.add_tokens(Balance::Cw20(Cw20CoinVerified {
            address: foo_token.clone(),
            amount: Uint128(23400),
        }));
        assert_eq!(
            tokens.cw20,
            vec![
                Cw20CoinVerified {
                    address: foo_token,
                    amount: Uint128(35745),
                },
                Cw20CoinVerified {
                    address: bar_token,
                    amount: Uint128(777),
                }
            ]
        );
    }

    #[test]
    fn all_escrow_ids_in_order() {
        let mut storage = MockStorage::new();
        ESCROWS
            .save(&mut storage, &"lazy", &dummy_escrow())
            .unwrap();
        ESCROWS
            .save(&mut storage, &"assign", &dummy_escrow())
            .unwrap();
        ESCROWS.save(&mut storage, &"zen", &dummy_escrow()).unwrap();

        let ids = all_escrow_ids(&storage).unwrap();
        assert_eq!(3, ids.len());
        assert_eq!(
            vec!["assign".to_string(), "lazy".to_string(), "zen".to_string()],
            ids
        )
    }
}
