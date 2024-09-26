use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Empty, StdResult, Uint128};
use cosmwasm_testing_util::{test_tube::FEE_DENOM, ExecuteResponse, MockResult};
use cw20::{BalanceResponse, Cw20Coin};
use cw721::OwnerOfResponse;
use cw_otc_common::{
    definitions::{OtcItem, OtcItemInfo, OtcPosition},
    msgs::{CreateOtcMsg, ExecuteOtcMsg, OtcItemRegistration},
};
use rhaki_cw_plus::{
    math::IntoUint,
    serde_value::{json, StdValue as Value},
    traits::IntoAddr,
};

use super::app_ext::{MergeCoin, TestMockApp};
pub type AppResult = MockResult<ExecuteResponse>;

#[cw_serde]
pub enum TokenType {
    Cw20,
    Native,
    Cw721,
}

#[derive(Debug)]
pub struct Def<'a> {
    pub addr_otc: Option<Addr>,
    pub code_id_cw20: Option<u64>,
    pub code_id_cw721: Option<u64>,
    pub fee_collector: &'a str,
    pub owner: &'a str,
    pub otc_fee: Vec<OtcItemInfo>,
}

impl<'a> Def<'a> {
    pub fn new(owner: &'a str, fee_collector: &'a str) -> Self {
        Self {
            addr_otc: None,
            code_id_cw20: None,
            code_id_cw721: None,
            fee_collector,
            owner,
            otc_fee: vec![OtcItemInfo::Token {
                denom: FEE_DENOM.to_string(),
                amount: 100_u128.into(),
            }],
        }
    }

    pub fn get_native_fee(&self) -> Vec<Coin> {
        self.otc_fee
            .iter()
            .filter_map(|fee| match fee {
                OtcItemInfo::Token { denom, amount } => Some(Coin::new(amount.u128(), denom)),
                _ => None,
            })
            .collect()
    }
}

pub fn startup(app: &mut TestMockApp, def: &mut Def) {
    let otc_code_id = app.upload(include_bytes!("./testdata/cw-otc.wasm"));
    let cw20_code_id = app.upload(include_bytes!("./testdata/cw20-base.wasm"));
    let cw721_code_id = app.upload(include_bytes!("./testdata/cw721-base.wasm"));

    def.code_id_cw20 = Some(cw20_code_id);
    def.code_id_cw721 = Some(cw721_code_id);

    let otc_addr = app
        .instantiate(
            otc_code_id,
            def.owner.into_unchecked_addr(),
            &cw_otc_common::msgs::InstantiateMsg {
                owner: def.owner.to_string(),
                fee: def.otc_fee.clone(),
                fee_collector: def.fee_collector.to_string(),
            },
            &[],
            "otc",
        )
        .unwrap();

    def.addr_otc = Some(otc_addr);
}

fn native_funds_from_otc_item_registration(items: &[OtcItemRegistration]) -> Vec<Coin> {
    items
        .iter()
        .filter_map(|item| {
            if let OtcItemInfo::Token { denom, amount } = &item.item_info {
                Some(Coin::new(amount.u128(), denom))
            } else {
                None
            }
        })
        .collect()
}

fn native_funds_from_otc_item(items: &[OtcItem]) -> Vec<Coin> {
    items
        .iter()
        .filter_map(|item| {
            if let OtcItemInfo::Token { denom, amount } = &item.item_info {
                Some(Coin::new(amount.u128(), denom))
            } else {
                None
            }
        })
        .collect()
}

pub fn create_token(
    app: &mut TestMockApp,
    def: &mut Def,
    token_name: &str,
    token_type: TokenType,
    initial_balance: Vec<(&str, &str)>,
) -> Addr {
    match token_type {
        TokenType::Cw20 => app
            .instantiate(
                def.code_id_cw20.unwrap(),
                def.owner.into_unchecked_addr(),
                &cw20_base::msg::InstantiateMsg {
                    name: token_name.to_string(),
                    symbol: token_name.to_string(),
                    decimals: 6,
                    initial_balances: initial_balance
                        .into_iter()
                        .map(|(to, amount)| Cw20Coin {
                            address: to.to_string(),
                            amount: amount.into_uint128(),
                        })
                        .collect(),
                    mint: Some(cw20::MinterResponse {
                        minter: def.owner.to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                &[],
                token_name,
            )
            .unwrap(),
        TokenType::Cw721 => {
            let addr = app
                .instantiate(
                    def.code_id_cw721.unwrap(),
                    def.owner.into_unchecked_addr(),
                    &cw721_base::msg::InstantiateMsg {
                        name: token_name.to_string(),
                        symbol: token_name.to_string(),
                        minter: def.owner.to_string(),
                    },
                    &[],
                    token_name,
                )
                .unwrap();

            for (to, token_id) in initial_balance {
                mint_token(app, def, to, (addr.as_str(), token_type.clone()), token_id)
            }

            addr
        }
        TokenType::Native => todo!(),
    }
}

pub fn mint_token(
    app: &mut TestMockApp,
    def: &mut Def,
    to: &str,
    token_info: (&str, TokenType),
    amount: &str,
) {
    match token_info.1 {
        TokenType::Cw20 => {
            app.execute(
                def.owner.into_unchecked_addr(),
                token_info.0.into_unchecked_addr(),
                &cw20_base::msg::ExecuteMsg::Mint {
                    recipient: to.to_string(),
                    amount: amount.into_uint128(),
                },
                &[],
            )
            .unwrap();
        }
        TokenType::Native => {
            app.send_coins(
                Addr::unchecked(def.owner),
                Addr::unchecked(to),
                &[Coin::new(amount.into_uint128().into(), token_info.0)],
            )
            .unwrap();
        }
        TokenType::Cw721 => {
            app.execute(
                def.owner.into_unchecked_addr(),
                token_info.0.into_unchecked_addr(),
                &cw721_base::ExecuteMsg::Mint::<Value, Empty> {
                    token_id: amount.to_string(),
                    owner: to.to_string(),
                    token_uri: None,
                    extension: json!({}),
                },
                &[],
            )
            .unwrap();
        }
    }
}

pub fn increase_allowance(
    app: &mut TestMockApp,
    sender: &str,
    to: &str,
    addr: &Addr,
    token_type: TokenType,
    amount: &str,
) {
    match token_type {
        TokenType::Cw20 => {
            app.execute(
                sender.into_unchecked_addr(),
                addr.clone(),
                &cw20::Cw20ExecuteMsg::IncreaseAllowance {
                    spender: to.to_string(),
                    amount: amount.into_uint128(),
                    expires: None,
                },
                &[],
            )
            .unwrap();
        }
        TokenType::Cw721 => {
            app.execute(
                sender.into_unchecked_addr(),
                addr.clone(),
                &cw721_base::ExecuteMsg::Approve::<Value, Empty> {
                    spender: to.to_string(),
                    token_id: amount.to_string(),
                    expires: None,
                },
                &[],
            )
            .unwrap();
        }
        TokenType::Native => todo!(),
    }
}

// run

pub fn run_create_otc(
    app: &mut TestMockApp,
    def: &mut Def,
    creator: &str,
    executor: &str,
    offer: &[OtcItemRegistration],
    ask: &[OtcItemRegistration],
    mut extra_coin: Vec<Coin>,
) -> AppResult {
    let mut coins = native_funds_from_otc_item_registration(offer);

    coins.append(&mut extra_coin);

    let coins = coins.merge();

    app.execute(
        creator.into_unchecked_addr(),
        def.addr_otc.clone().unwrap(),
        &cw_otc_common::msgs::ExecuteMsg::CreateOtc(CreateOtcMsg {
            executor: Some(executor.to_string()),
            offer: offer.to_vec(),
            ask: ask.to_vec(),
        }),
        &coins,
    )
}

pub fn run_execute_otc(
    app: &mut TestMockApp,
    def: &mut Def,
    sender: &str,
    id: u64,
    mut extra_coin: Vec<Coin>,
) -> AppResult {
    let position = qy_otc_active_position(app, def, id).unwrap();

    let mut coins = native_funds_from_otc_item(&position.ask);

    coins.append(&mut extra_coin);

    let coins = coins.merge();
    app.execute(
        sender.into_unchecked_addr(),
        def.addr_otc.clone().unwrap(),
        &cw_otc_common::msgs::ExecuteMsg::ExecuteOtc(ExecuteOtcMsg { id }),
        &coins,
    )
}

// queries

pub fn qy_otc_active_position(app: &TestMockApp, def: &Def, id: u64) -> StdResult<OtcPosition> {
    app.query(
        def.addr_otc.clone().unwrap(),
        &cw_otc_common::msgs::QueryMsg::Position { id },
    )
}

pub fn qy_otc_executed_position(app: &TestMockApp, def: &Def, id: u64) -> StdResult<OtcPosition> {
    app.query(
        def.addr_otc.clone().unwrap(),
        &cw_otc_common::msgs::QueryMsg::Position { id },
    )
}

pub fn qy_balance_native(app: &TestMockApp, denom: &str, user: &str) -> Uint128 {
    app.query_balance(Addr::unchecked(user), denom.to_string())
        .unwrap()
}

pub fn qy_balance_cw20(app: &TestMockApp, addr: &Addr, user: &str) -> Uint128 {
    let ret: BalanceResponse = app
        .query(
            addr.clone(),
            &cw20::Cw20QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap();
    ret.balance
}

pub fn qy_balance_nft(app: &TestMockApp, addr: &Addr, token_id: &str, user: &str) -> bool {
    let ret: OwnerOfResponse = app
        .query(
            Addr::unchecked(addr),
            &cw721::Cw721QueryMsg::OwnerOf {
                token_id: token_id.to_string(),
                include_expired: None,
            },
        )
        .unwrap();

    ret.owner == *user
}
