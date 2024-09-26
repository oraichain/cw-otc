use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use cosmwasm_schema::schemars::JsonSchema;
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_std::{Binary, Coin, CustomQuery, Deps, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_multi_test::ContractWrapper;

type ContractFn<T, C, E, Q> =
    fn(deps: DepsMut<Q>, env: Env, info: MessageInfo, msg: T) -> Result<Response<C>, E>;

type QueryFn<T, E, Q> = fn(deps: Deps<Q>, env: Env, msg: T) -> Result<Binary, E>;

pub fn create_code<
    T1: DeserializeOwned + fmt::Debug + 'static,
    T2: DeserializeOwned + 'static,
    T3: DeserializeOwned + 'static,
    C: Clone + fmt::Debug + PartialEq + JsonSchema + 'static,
    E1: Display + fmt::Debug + Send + Sync + 'static,
    E2: Display + fmt::Debug + Send + Sync + 'static,
    E3: Display + fmt::Debug + Send + Sync + 'static,
    Q: CustomQuery + DeserializeOwned + 'static,
>(
    instantiate: ContractFn<T2, C, E2, Q>,
    execute: ContractFn<T1, C, E1, Q>,
    query: QueryFn<T3, E3, Q>,
) -> Box<ContractWrapper<T1, T2, T3, E1, E2, E3, C, Q>> {
    let contract: ContractWrapper<T1, T2, T3, E1, E2, E3, C, Q> =
        ContractWrapper::new(execute, instantiate, query);

    Box::new(contract)
}

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
