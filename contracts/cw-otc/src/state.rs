use cw_otc_common::definitions::{Config, OtcPosition};
use cw_storage_plus::{index_list, IndexedMap, Item, MultiIndex};

pub const CONFIG: Item<Config> = Item::new("config");

pub type PositionMap<'a> = IndexedMap<'a, u64, OtcPosition, OtcPositionIndexer<'a>>;

#[index_list(OtcPosition)]
pub struct OtcPositionIndexer<'a> {
    pub owner: MultiIndex<'a, String, OtcPosition, u64>,
    pub executor: MultiIndex<'a, String, OtcPosition, u64>,
    pub owner_executor: MultiIndex<'a, (String, String), OtcPosition, u64>,
    pub status: MultiIndex<'a, String, OtcPosition, u64>,
    pub owner_status: MultiIndex<'a, (String, String), OtcPosition, u64>,
    pub executor_status: MultiIndex<'a, (String, String), OtcPosition, u64>,
    pub owner_executor_status: MultiIndex<'a, (String, String, String), OtcPosition, u64>,
}

pub fn positions<'a>() -> PositionMap<'a> {
    let indexer = OtcPositionIndexer {
        owner: MultiIndex::new(
            |_, val| val.owner.to_string(),
            "active_position",
            "active_position_owner",
        ),
        executor: MultiIndex::new(
            |_, val| {
                val.executor
                    .clone()
                    .map(|val| val.to_string())
                    .unwrap_or("".to_string())
            },
            "active_position",
            "active_position_executor",
        ),
        owner_executor: MultiIndex::new(
            |_, val| {
                (
                    val.owner.to_string(),
                    val.executor
                        .clone()
                        .map(|val| val.to_string())
                        .unwrap_or("".to_string()),
                )
            },
            "active_position",
            "active_position_owner_executor",
        ),
        status: MultiIndex::new(
            |_, val| val.status.as_string_ref(),
            "active_position",
            "active_position_status",
        ),
        owner_status: MultiIndex::new(
            |_, val| (val.owner.to_string(), val.status.as_string_ref()),
            "active_position",
            "active_position_owner_status",
        ),
        executor_status: MultiIndex::new(
            |_, val| {
                (
                    val.executor
                        .clone()
                        .map(|val| val.to_string())
                        .unwrap_or("".to_string()),
                    val.status.as_string_ref(),
                )
            },
            "active_position",
            "active_position_executor_status",
        ),
        owner_executor_status: MultiIndex::new(
            |_, val| {
                (
                    val.owner.to_string(),
                    val.executor
                        .clone()
                        .map(|val| val.to_string())
                        .unwrap_or("".to_string()),
                    val.status.as_string_ref(),
                )
            },
            "active_position",
            "active_position_owner_executor_status",
        ),
    };

    IndexedMap::new("active_position", indexer)
}
