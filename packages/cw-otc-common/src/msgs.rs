use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Order;

use super::definitions::{OtcItemInfo, OtcPosition};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub fee: Vec<OtcItemInfo>,
    pub fee_collector: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateOtc(CreateOtcMsg),
    ExecuteOtc(ExecuteOtcMsg),
    ClaimOtc(ClaimOtcMsg),
    CancelOtc(CancelOtcMsg),
}

#[cw_serde]
pub struct CreateOtcMsg {
    pub executor: Option<String>,
    pub offer: Vec<OtcItemRegistration>,
    pub ask: Vec<OtcItemRegistration>,
}

#[cw_serde]
pub struct ExecuteOtcMsg {
    pub id: u64,
}

#[cw_serde]
pub struct ClaimOtcMsg {
    pub id: u64,
}

#[cw_serde]
pub struct CancelOtcMsg {
    pub id: u64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(OtcPosition)]
    Position { id: u64 },
    #[returns(Vec<OtcPosition>)]
    Positions {
        limit: Option<u32>,
        start_after: Option<u64>,
        filters: Option<QueryPositionsFilter>,
        order: Option<QueryPositionsFilterOrder>,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct VestingInfoRegistration {
    pub cliff: Option<u64>,
    pub vesting: Option<u64>,
}

#[cw_serde]
pub struct OtcItemRegistration {
    pub item_info: OtcItemInfo,
    pub vesting: Option<VestingInfoRegistration>,
}

#[cw_serde]
pub struct QueryPositionsFilter {
    pub owner: Option<String>,
    pub executor: Option<String>,
    pub status: Option<QueryPositionsFilterStatus>,
}

#[cw_serde]
pub enum QueryPositionsFilterStatus {
    Vesting,
    Pending,
    Executed,
}

impl QueryPositionsFilterStatus {
    pub fn as_string(&self) -> String {
        match self {
            QueryPositionsFilterStatus::Vesting => "vesting".to_string(),
            QueryPositionsFilterStatus::Pending => "pending".to_string(),
            QueryPositionsFilterStatus::Executed => "executed".to_string(),
        }
    }
}

#[cw_serde]
pub enum QueryPositionsFilterOrder {
    Ascending,
    Descending,
}

impl From<QueryPositionsFilterOrder> for Order {
    fn from(val: QueryPositionsFilterOrder) -> Self {
        match val {
            QueryPositionsFilterOrder::Ascending => Order::Ascending,
            QueryPositionsFilterOrder::Descending => Order::Descending,
        }
    }
}
