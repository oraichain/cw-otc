use cosmwasm_std::{Deps, Order, StdError, StdResult};
use cw_otc_common::{
    definitions::OtcPosition,
    msgs::{QueryPositionsFilter, QueryPositionsFilterOrder},
};

use crate::{
    functions::{get_items, get_multi_index_values},
    state::positions,
};

pub fn qy_position(deps: Deps, id: u64) -> StdResult<OtcPosition> {
    positions().load(deps.storage, id)
}

pub fn qy_positions(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
    filters: Option<QueryPositionsFilter>,
    order: Option<QueryPositionsFilterOrder>,
) -> StdResult<Vec<OtcPosition>> {
    let order: Order = order
        .unwrap_or(QueryPositionsFilterOrder::Descending)
        .into();
    if let Some(filters) = filters {
        match (filters.owner, filters.executor, filters.status) {
            (None, None, None) => return Err(StdError::generic_err("None filter provided")),
            // status
            (None, None, Some(status)) => get_multi_index_values(
                deps.storage,
                status.as_string(),
                positions().idx.status,
                order,
                start_after,
                limit,
            ),
            // executor
            (None, Some(executor), None) => get_multi_index_values(
                deps.storage,
                executor,
                positions().idx.executor,
                order,
                start_after,
                limit,
            ),
            // executor-status
            (None, Some(executor), Some(status)) => get_multi_index_values(
                deps.storage,
                (executor, status.as_string()),
                positions().idx.executor_status,
                order,
                start_after,
                limit,
            ),
            // owner
            (Some(owner), None, None) => get_multi_index_values(
                deps.storage,
                owner,
                positions().idx.owner,
                order,
                start_after,
                limit,
            ),
            // owner-status
            (Some(owner), None, Some(status)) => get_multi_index_values(
                deps.storage,
                (owner, status.as_string()),
                positions().idx.owner_status,
                order,
                start_after,
                limit,
            ),
            // owner-executor
            (Some(owner), Some(executor), None) => get_multi_index_values(
                deps.storage,
                (owner, executor),
                positions().idx.owner_executor,
                order,
                start_after,
                limit,
            ),
            // owner-executor-status
            (Some(owner), Some(executor), Some(status)) => get_multi_index_values(
                deps.storage,
                (owner, executor, status.as_string()),
                positions().idx.owner_executor_status,
                order,
                start_after,
                limit,
            ),
        }
    } else {
        get_items(deps.storage, positions(), order, limit, start_after)
    }
    .map(|val| val.into_iter().map(|(_, val)| val).collect())
}
