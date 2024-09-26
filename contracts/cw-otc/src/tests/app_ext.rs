use cosmwasm_std::{Coin, Uint128};
use std::collections::HashMap;

pub trait MergeCoin {
    fn merge(self) -> Vec<Coin>;
}

impl MergeCoin for Vec<Coin> {
    fn merge(self) -> Vec<Coin> {
        let mut map: HashMap<String, Uint128> = HashMap::new();

        for i in self {
            if let Some(val) = map.get_mut(&i.denom) {
                *val += i.amount
            } else {
                map.insert(i.denom.to_string(), i.amount);
            }
        }

        map.into_iter()
            .map(|(denom, amount)| Coin::new(amount.into(), denom))
            .collect()
    }
}

pub type TestMockApp = cosmwasm_testing_util::TestTubeMockApp;
