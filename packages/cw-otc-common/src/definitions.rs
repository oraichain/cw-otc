use std::cmp::min;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    Addr, BankMsg, Coin, CosmosMsg, Decimal, Deps, Env, StdError, StdResult, Uint128, WasmMsg,
};
use rhaki_cw_plus::{traits::IntoAddr, wasm::WasmMsgBuilder};

use super::msgs::{CreateOtcMsg, OtcItemRegistration, VestingInfoRegistration};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub counter_otc: u64,
    pub fee: Vec<OtcItemInfo>,
    pub fee_collector: Addr,
}

impl Config {
    pub fn new(
        deps: Deps,
        owner: Addr,
        fee: Vec<OtcItemInfo>,
        fee_collector: Addr,
    ) -> StdResult<Config> {
        for i in &fee {
            i.validate(deps)?;
        }

        Ok(Config {
            owner,
            counter_otc: 0,
            fee,
            fee_collector,
        })
    }
}

#[cw_serde]
pub struct OtcItem {
    pub item_info: OtcItemInfo,
    pub vesting_info: Option<VestingInfo>,
}

impl OtcItem {
    pub fn validate(&self, deps: Deps) -> StdResult<()> {
        if let Some(vesting) = &self.vesting_info {
            vesting.validate()?
        }
        self.item_info.validate(deps)
    }

    pub fn sendable_amount_and_update_claimed_amount(
        &mut self,
        env: &Env,
        position_status: &OtcPositionStatus,
    ) -> StdResult<Uint128> {
        match &mut self.vesting_info {
            Some(vesting_info) => {
                let max_amount = self.item_info.get_amount();
                let vesting_start = position_status.get_vesting_start()?;
                let mut delta = env.block.time.seconds() - vesting_start;

                let cliff_window = if let Some(cliff) = vesting_info.cliff {
                    if cliff >= delta {
                        return Ok(Uint128::zero());
                    }
                    cliff
                } else {
                    0
                };

                delta -= cliff_window;

                let unchecked_claimable_amount = if let Some(vesting_window) = vesting_info.vesting
                {
                    max_amount * Decimal::from_ratio(min(delta, vesting_window), vesting_window)
                } else {
                    max_amount
                };

                let claimabile_amount = unchecked_claimable_amount - vesting_info.claimed;

                vesting_info.claimed += claimabile_amount;

                Ok(claimabile_amount)
            }
            None => Ok(self.item_info.get_amount()),
        }
    }
}

impl From<OtcItemRegistration> for OtcItem {
    fn from(value: OtcItemRegistration) -> Self {
        OtcItem {
            item_info: value.item_info,
            vesting_info: value.vesting.map(|val| val.into()),
        }
    }
}

#[cw_serde]
pub struct VestingInfo {
    pub cliff: Option<u64>,
    pub vesting: Option<u64>,
    pub claimed: Uint128,
}

impl VestingInfo {
    pub fn validate(&self) -> StdResult<()> {
        if self.cliff.is_none() && self.vesting.is_none() {
            return Err(StdError::generic_err(
                "VestingInfo must have a vesting or cliff info",
            ));
        }

        if let Some(vesting) = self.vesting {
            if vesting == 0 {
                return Err(StdError::generic_err("Vesting must be > 0"));
            }
        }

        if let Some(cliff) = self.cliff {
            if cliff == 0 {
                return Err(StdError::generic_err("Cliff must be > 0"));
            }
        }

        Ok(())
    }
}

impl From<VestingInfoRegistration> for VestingInfo {
    fn from(value: VestingInfoRegistration) -> Self {
        VestingInfo {
            cliff: value.cliff,
            vesting: value.vesting,
            claimed: Uint128::zero(),
        }
    }
}

#[cw_serde]
pub enum OtcItemInfo {
    Token { denom: String, amount: Uint128 },
    Cw20 { contract: Addr, amount: Uint128 },
    Cw721 { contract: Addr, token_id: String },
}

impl OtcItemInfo {
    pub fn validate(&self, deps: Deps) -> StdResult<()> {
        match self {
            OtcItemInfo::Token { .. } => Ok(()),
            OtcItemInfo::Cw20 { contract, .. } => {
                contract.to_string().into_addr(deps.api).map(|_| ())
            }
            OtcItemInfo::Cw721 { contract, .. } => {
                contract.to_string().into_addr(deps.api).map(|_| ())
            }
        }
    }

    pub fn get_amount(&self) -> Uint128 {
        match self {
            OtcItemInfo::Token { amount, .. } => *amount,
            OtcItemInfo::Cw20 { amount, .. } => *amount,
            OtcItemInfo::Cw721 { .. } => Uint128::one(),
        }
    }

    pub fn build_send_msg(
        &self,
        env: &Env,
        sender: &Addr,
        to: &Addr,
        override_amount: Option<Uint128>,
    ) -> StdResult<CosmosMsg> {
        if let Some(override_amount) = override_amount {
            if override_amount == Uint128::zero() {
                return Err(StdError::generic_err("Invalid 0 amount"));
            }
        }
        match self {
            OtcItemInfo::Token { denom, amount } => {
                if env.contract.address != sender {
                    return Err(StdError::generic_err(
                        "Sender for native token must be the contract itself",
                    ));
                }

                Ok(BankMsg::Send {
                    to_address: to.to_string(),
                    amount: vec![Coin::new(override_amount.unwrap_or(*amount).u128(), denom)],
                }
                .into())
            }
            OtcItemInfo::Cw20 { contract, amount } => Ok(WasmMsg::build_execute(
                contract,
                if env.contract.address == sender {
                    cw20::Cw20ExecuteMsg::Transfer {
                        recipient: to.to_string(),
                        amount: override_amount.unwrap_or(*amount).to_owned(),
                    }
                } else {
                    cw20::Cw20ExecuteMsg::TransferFrom {
                        owner: sender.to_string(),
                        recipient: to.to_string(),
                        amount: override_amount.unwrap_or(*amount).to_owned(),
                    }
                },
                vec![],
            )?
            .into()),
            OtcItemInfo::Cw721 { contract, token_id } => Ok(WasmMsg::build_execute(
                contract,
                cw721::Cw721ExecuteMsg::TransferNft {
                    recipient: to.to_string(),
                    token_id: token_id.to_owned(),
                },
                vec![],
            )?
            .into()),
        }
    }
}

#[cw_serde]
pub struct OtcPosition {
    pub id: u64,
    pub owner: Addr,
    pub executor: Option<Addr>,
    pub offer: Vec<OtcItem>,
    pub ask: Vec<OtcItem>,
    pub creation_time: u64,
    pub status: OtcPositionStatus,
}

impl OtcPosition {
    pub fn validate(&self, deps: Deps) -> StdResult<()> {
        if let Some(executor) = &self.executor {
            executor.to_string().into_addr(deps.api)?;
        }

        for item in self.offer.iter().chain(self.ask.iter()) {
            item.validate(deps)?;
        }

        Ok(())
    }
    pub fn from_create_otc_msg(
        deps: Deps,
        env: &Env,
        msg: CreateOtcMsg,
        id: u64,
        owner: Addr,
    ) -> StdResult<OtcPosition> {
        Ok(OtcPosition {
            id,
            owner,
            executor: msg
                .executor
                .map(|val| val.into_addr(deps.api))
                .transpose()?,
            offer: msg.offer.into_iter().map(|val| val.into()).collect(),
            ask: msg.ask.into_iter().map(|val| val.into()).collect(),
            creation_time: env.block.time.seconds(),
            status: OtcPositionStatus::Pending,
        })
    }

    pub fn active(&mut self, env: &Env, executor: &Addr) -> StdResult<()> {
        if let Some(saved_executor) = &self.executor {
            if saved_executor != executor {
                return Err(StdError::generic_err("Unauthorized"));
            }
        } else {
            self.executor = Some(executor.clone())
        };

        match self.status {
            OtcPositionStatus::Pending => {
                self.status = OtcPositionStatus::Vesting(env.block.time.seconds())
            }
            _ => return Err(StdError::generic_err("Active require status in Pending")),
        }

        Ok(())
    }

    pub fn try_close(&mut self, env: &Env) -> StdResult<()> {
        if let OtcPositionStatus::Vesting(..) = self.status {
            let all_items: Vec<OtcItem> = self
                .ask
                .clone()
                .into_iter()
                .chain(self.offer.clone().into_iter())
                .collect();

            for item in all_items {
                if let Some(vesting_info) = item.vesting_info {
                    if vesting_info.claimed != item.item_info.get_amount() {
                        return Ok(());
                    }
                }
            }

            self.status = OtcPositionStatus::Executed(env.block.time.seconds())
        } else {
            return Err(StdError::generic_err("Try_close require status in Vesting"));
        }

        Ok(())
    }
}

#[cw_serde]
pub enum OtcPositionStatus {
    Pending,
    Vesting(u64),
    Executed(u64),
}

impl OtcPositionStatus {
    pub fn get_vesting_start(&self) -> StdResult<u64> {
        match self {
            OtcPositionStatus::Vesting(val) => Ok(*val),
            _ => Err(StdError::generic_err("OtcPositionStatus is not vesting")),
        }
    }

    pub fn is_in_pending(&self) -> bool {
        matches!(self, OtcPositionStatus::Pending)
    }

    pub fn as_string_ref(&self) -> String {
        match self {
            OtcPositionStatus::Pending => "pending".to_string(),
            OtcPositionStatus::Vesting(_) => "vesting".to_string(),
            OtcPositionStatus::Executed(_) => "executed".to_string(),
        }
    }
}
