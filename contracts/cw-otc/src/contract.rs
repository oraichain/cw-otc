use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use cw_otc_common::{
    definitions::Config,
    msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};

use crate::{
    execute::{run_cancel_otc, run_claim_otc, run_create_otc, run_execute_otc},
    query::{qy_position, qy_positions},
    response::ContractResponse,
    state::CONFIG,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResponse {
    let config = Config::new(
        deps.as_ref(),
        deps.api.addr_validate(&msg.owner)?,
        msg.fee,
        deps.api.addr_validate(&msg.fee_collector)?,
    )?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", msg.owner))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResponse {
    match msg {
        ExecuteMsg::CreateOtc(msg) => run_create_otc(deps, env, info, msg),
        ExecuteMsg::ExecuteOtc(msg) => run_execute_otc(deps, env, info, msg),
        ExecuteMsg::ClaimOtc(msg) => run_claim_otc(deps, env, info, msg),
        ExecuteMsg::CancelOtc(msg) => run_cancel_otc(deps, env, info, msg),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Position { id } => to_json_binary(&qy_position(deps, id)?),
        QueryMsg::Positions {
            limit,
            start_after,
            filters,
            order,
        } => to_json_binary(&qy_positions(deps, start_after, limit, filters, order)?),
    }
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResponse {
    Ok(Response::default())
}
