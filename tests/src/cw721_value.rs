use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use cw721_base::{ContractError, Cw721Contract, ExecuteMsg, InstantiateMsg, QueryMsg};
use rhaki_cw_plus::serde_value::StdValue as Value;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let tract = Cw721Contract::<Value, Empty, Empty, Empty>::default();
    tract.instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<Value, Empty>,
) -> Result<Response, ContractError> {
    let tract = Cw721Contract::<Value, Empty, Empty, Empty>::default();
    tract.execute(deps, env, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg<Empty>) -> StdResult<Binary> {
    let tract = Cw721Contract::<Value, Empty, Empty, Empty>::default();
    tract.query(deps, env, msg)
}
