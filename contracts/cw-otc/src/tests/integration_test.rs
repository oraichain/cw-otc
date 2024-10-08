use cosmwasm_std::Coin;
use cosmwasm_testing_util::test_tube::FEE_DENOM;
use cw_otc_common::{definitions::OtcItemInfo, msgs::OtcItemRegistration};

use crate::tests::app_ext::TestMockApp;

use super::helper::{
    create_token, increase_allowance, mint_token, qy_balance_cw20, qy_balance_native,
    qy_balance_nft, qy_otc_active_position, qy_otc_executed_position, run_create_otc,
    run_execute_otc, startup, Def, TokenType,
};

#[test]
#[rustfmt::skip]
pub fn test() {
    
    let (mut app, accounts) = TestMockApp::new(&[
        ("owner",&[Coin::new(100_000_000_000u128, FEE_DENOM)]),
        ("creator",&[Coin::new(100_000_000_000u128, FEE_DENOM)]),
        ("executor",&[Coin::new(100_000_000_000u128, FEE_DENOM)]),
        ("fee_collector",&[Coin::new(100_000_000_000u128, FEE_DENOM)]),
    ]);
    
    let mut def = Def::new(&accounts[0],&accounts[1]);
    
    startup(&mut app,&mut def);

    let creator = &accounts[2];
    let executor = &accounts[3];

    let fee = def.get_native_fee();
   
    // Create tokens

    let offer_nft_id = "1";
    let offer_cw20_amount= 100_u128;
    let offer_native_amount= 150_u128;

    let offer_nft_addr = create_token(&mut app, &mut def, "NftOffer", TokenType::Cw721, vec![(creator, offer_nft_id)]);
    let offer_cw20_addr = create_token(&mut app, &mut def, "TokenOffer", TokenType::Cw20, vec![(creator, &offer_cw20_amount.to_string())]);
    
    mint_token(&mut app, &mut def, creator, (FEE_DENOM, TokenType::Native), &offer_native_amount.to_string());

    let ask_nft_id = "2";
    let ask_cw20_amount= 200_u128;
    let ask_native_amount= 250_u128;

    let ask_nft_addr = create_token(&mut app, &mut def, "NftOffer", TokenType::Cw721, vec![(executor, ask_nft_id)]);
    let ask_cw20_addr = create_token(&mut app, &mut def, "TokenOffer", TokenType::Cw20, vec![(executor, &ask_cw20_amount.to_string())]);
    
    mint_token(&mut app, &mut def, executor, (FEE_DENOM, TokenType::Native), &ask_native_amount.to_string());

    // Increase allowance

    increase_allowance(&mut app, creator, def.addr_otc.clone().unwrap().as_ref(), &offer_nft_addr, TokenType::Cw721, offer_nft_id);
    increase_allowance(&mut app, creator, def.addr_otc.clone().unwrap().as_ref(), &offer_cw20_addr, TokenType::Cw20, &offer_cw20_amount.to_string());

    // Create otc

    let offer_items = vec![
        OtcItemRegistration { item_info: OtcItemInfo::Token { denom: FEE_DENOM.to_string(), amount: offer_native_amount.into() }, vesting: None },
        OtcItemRegistration { item_info: OtcItemInfo::Cw20 { contract: offer_cw20_addr.clone(), amount: offer_cw20_amount.into() }, vesting: None },
        OtcItemRegistration { item_info: OtcItemInfo::Cw721 { contract: offer_nft_addr.clone(), token_id: offer_nft_id.to_string() }, vesting: None }
    ];

    let ask_items = vec![
        OtcItemRegistration { item_info: OtcItemInfo::Token { denom: FEE_DENOM.to_string(), amount: ask_native_amount.into() }, vesting: None },
        OtcItemRegistration { item_info: OtcItemInfo::Cw20 { contract: ask_cw20_addr.clone(), amount: ask_cw20_amount.into() }, vesting: None },
        OtcItemRegistration { item_info: OtcItemInfo::Cw721 { contract: ask_nft_addr.clone(), token_id: ask_nft_id.to_string() }, vesting: None }
    ];

    // fails for missing fee

    run_create_otc(&mut app, &mut def, creator, executor, &offer_items, &ask_items, vec![]).unwrap_err();
    mint_token(&mut app, &mut def, creator, (&fee[0].denom, TokenType::Native), &fee[0].amount.to_string());
    run_create_otc(&mut app, &mut def, creator, executor, &offer_items, &ask_items, fee.clone()).unwrap();

    // assert position

    assert_eq!(offer_cw20_amount, qy_balance_cw20(&app, &offer_cw20_addr, def.addr_otc.clone().unwrap().as_ref()).u128());
    assert_eq!(offer_native_amount, qy_balance_native(&app, FEE_DENOM, def.addr_otc.clone().unwrap().as_ref()).u128());
    assert!(qy_balance_nft(&app, &offer_nft_addr, offer_nft_id, def.addr_otc.clone().unwrap().as_ref()));

    // close position

    increase_allowance(&mut app, executor, def.addr_otc.clone().unwrap().as_ref(), &ask_nft_addr, TokenType::Cw721, ask_nft_id);
    increase_allowance(&mut app, executor, def.addr_otc.clone().unwrap().as_ref(), &ask_cw20_addr, TokenType::Cw20, &ask_cw20_amount.to_string());
    mint_token(&mut app, &mut def, executor, (&fee[0].denom, TokenType::Native), &fee[0].amount.to_string());

    

    run_execute_otc(&mut app, &mut def, executor, 1, vec![]).unwrap_err();
    run_execute_otc(&mut app, &mut def, executor, 1, fee.clone()).unwrap();

    // assert result
    assert_eq!(offer_cw20_amount, qy_balance_cw20(&app, &offer_cw20_addr, executor).u128());
    assert_eq!(96803880150, qy_balance_native(&app, FEE_DENOM, executor).u128());
    assert!(qy_balance_nft(&app, &offer_nft_addr, offer_nft_id, executor));

    assert_eq!(ask_cw20_amount, qy_balance_cw20(&app, &ask_cw20_addr, creator).u128());
    assert_eq!(97823515250, qy_balance_native(&app, FEE_DENOM, creator).u128());
    assert!(qy_balance_nft(&app, &ask_nft_addr, ask_nft_id, creator));

    assert_eq!(100000000200, qy_balance_native(&app, &fee[0].denom, def.fee_collector).u128());

    qy_otc_executed_position(&app, &def, 1).unwrap();
    qy_otc_active_position(&app, &def, 1).unwrap();

}
