use std::str::FromStr;

use crate::{
    contract::{execute, instantiate, query, reply},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};

use anyhow::Result as AnyResult;
use bs721_account::{
    msg::SudoParams as Bs721SudoParams, ExecuteMsg as Bs721AccountExecuteMsg,
    QueryMsg as Bs721AccountQueryMsg,
};
use bs721_account_marketplace::helpers::get_char_price;
use bs_std::NATIVE_DENOM;
use btsg_account::{
    common::SECONDS_PER_YEAR,
    market::{
        state::{Bid, SudoParams as ProfileMarketplaceParams},
        ExecuteMsg as MarketplaceExecuteMsg, QueryMsg as MarketplaceQueryMsg,
        QueryMsg as ProfileMarketplaceQueryMsg, SudoMsg as MarketplaceSudoMsg,
    },
    minter::{
        BsAccountMinterQueryMsg, SudoParams as NameMinterParams, PUBLIC_MINT_START_TIME_IN_SECONDS,
    },
    Metadata,
};
use cosmwasm_std::{coins, Addr, Decimal, Empty, Timestamp, Uint128};
use cw721::{NumTokensResponse, OwnerOfResponse};
use cw_multi_test::{
    App as TestApp, AppResponse, BankSudo, Contract, ContractWrapper, Executor,
    SudoMsg as CwSudoMsg,
};

pub fn contract_minter() -> Box<dyn Contract<Empty, Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
    Box::new(contract)
}

pub fn contract_marketplace() -> Box<dyn Contract<Empty, Empty>> {
    let contract = ContractWrapper::new(
        bs721_account_marketplace::contract::execute,
        bs721_account_marketplace::contract::instantiate,
        bs721_account_marketplace::contract::query,
    )
    .with_sudo(bs721_account_marketplace::contract::sudo);
    Box::new(contract)
}

pub fn contract_collection() -> Box<dyn Contract<Empty, Empty>> {
    let contract = ContractWrapper::new(
        bs721_account::execute,
        bs721_account::instantiate,
        bs721_account::query,
    )
    .with_sudo(bs721_account::sudo::sudo);
    Box::new(contract)
}

//
pub fn contract_nft() -> Box<dyn Contract<Empty, Empty>> {
    let contract = ContractWrapper::new(
        bs721_base::entry::execute,
        bs721_base::entry::instantiate,
        bs721_base::entry::query,
    );
    Box::new(contract)
}

const USER: &str = "user";
const USER2: &str = "user2";
const USER3: &str = "user3";
const USER4: &str = "user4";
const BIDDER: &str = "bidder";
const BIDDER2: &str = "bidder2";
const ADMIN: &str = "admin";
const ADMIN2: &str = "admin2";
const NAME: &str = "bobo";
const NAME2: &str = "mccool";
const VERIFIER: &str = "verifier";
const OPERATOR: &str = "operator";

const TRADING_FEE_BPS: u64 = 200; // 2%
const BASE_PRICE: u128 = 100_000_000;
const BID_AMOUNT: u128 = 1_000_000_000;
const PER_ADDRESS_LIMIT: u32 = 2;
const TRADING_START_TIME_OFFSET_IN_SECONDS: u64 = 2 * SECONDS_PER_YEAR;

const MKT: &str = "contract0";
const MINTER: &str = "contract1";
const COLLECTION: &str = "contract2";
const WHITELIST: &str = "contract3";

// NOTE: This are mostly Marketplace integration tests. They could possibly be moved into the marketplace contract.

pub fn custom_mock_app(start_time: Option<Timestamp>) -> TestApp {
    let time = start_time.unwrap_or(PUBLIC_MINT_START_TIME_IN_SECONDS);
    set_block_time(TestApp::default(), time)
}

pub fn set_block_time(mut app: TestApp, time: Timestamp) -> TestApp {
    let mut block_info = app.block_info();
    block_info.time = time;
    app.set_block(block_info);
    app
}

// 1. Instantiate Name Marketplace
// 2. Instantiate Name Minter (which instantiates Name Collection)
// 3. Setup Name Marketplace with Name Minter and Collection addresses
// 4. Instantiate Whitelist
// 5. Update Whitelist with Name Minter
// 6. Add Whitelist to Name Minter
fn instantiate_contracts(
    creator: Option<String>,
    admin: Option<String>,
    start_time: Option<Timestamp>,
) -> TestApp {
    let mut app = custom_mock_app(start_time);
    let mkt_id = app.store_code(contract_marketplace());
    let minter_id = app.store_code(contract_minter());
    let sg721_id = app.store_code(contract_collection());
    // let wl_id = app.store_code(contract_whitelist());

    // 1. Instantiate Name Marketplace
    let msg = btsg_account::market::InstantiateMsg {
        trading_fee_bps: TRADING_FEE_BPS,
        min_price: Uint128::from(5u128),
        ask_interval: 60,
        max_renewals_per_block: 20,
        valid_bid_query_limit: 10,
        renew_window: 60 * 60 * 24 * 30,
        renewal_bid_percentage: Decimal::from_str("0.005").unwrap(),
        operator: OPERATOR.to_string(),
        // factory: todo!(),
        // collection: todo!(),
    };
    let marketplace = app
        .instantiate_contract(
            mkt_id,
            Addr::unchecked(creator.clone().unwrap_or_else(|| ADMIN.to_string())),
            &msg,
            &[],
            "Name-Marketplace",
            admin.clone(),
        )
        .unwrap();

    // 2. Instantiate Name Minter (which instantiates Name Collection)
    let msg = InstantiateMsg {
        admin: admin.clone(),
        verifier: Some(VERIFIER.to_string()),
        collection_code_id: sg721_id,
        marketplace_addr: marketplace.to_string(),
        base_price: Uint128::from(BASE_PRICE),
        min_name_length: 3,
        max_name_length: 63,
    };
    let minter = app
        .instantiate_contract(
            minter_id,
            Addr::unchecked(creator.unwrap_or_else(|| ADMIN2.to_string())),
            &msg,
            &[],
            "Name-Minter",
            None,
        )
        .unwrap();

    let bs721_collection: Addr = app
        .wrap()
        .query_wasm_smart(minter.clone(), &BsAccountMinterQueryMsg::Collection {})
        .unwrap();

    let msg = btsg_account::market::ExecuteMsg::Setup {
        minter: minter.to_string(),
        collection: bs721_collection.to_string(),
    };
    let res = app.execute_contract(
        Addr::unchecked(ADMIN.to_string()),
        marketplace.clone(),
        &msg,
        &[],
    );
    assert!(res.is_ok());

    let res: Addr = app
        .wrap()
        .query_wasm_smart(COLLECTION, &(Bs721AccountQueryMsg::AccountMarketplace {}))
        .unwrap();
    assert_eq!(res, marketplace.to_string());

    app
}

fn owner_of(app: &TestApp, token_id: String) -> String {
    let res: OwnerOfResponse = app
        .wrap()
        .query_wasm_smart(
            COLLECTION,
            &(bs721_base::msg::QueryMsg::<Empty>::OwnerOf {
                token_id,
                include_expired: None,
            }),
        )
        .unwrap();

    res.owner
}

fn update_block_time(app: &mut TestApp, add_secs: u64) {
    let mut block = app.block_info();
    block.time = block.time.plus_seconds(add_secs);
    app.set_block(block);
}

fn mint_and_list(app: &mut TestApp, name: &str, user: &str) -> AnyResult<AppResponse> {
    // set approval for user, for all tokens
    // approve_all is needed because we don't know the token_id before-hand
    let approve_all_msg: Bs721AccountExecuteMsg = Bs721AccountExecuteMsg::ApproveAll {
        operator: MKT.to_string(),
        expires: None,
    };
    let res = app.execute_contract(
        Addr::unchecked(user),
        Addr::unchecked(COLLECTION),
        &approve_all_msg,
        &[],
    );
    assert!(res.is_ok());

    let amount: Uint128 = (match name.len() {
        0..=2 => BASE_PRICE,
        3 => BASE_PRICE * 100,
        4 => BASE_PRICE * 10,
        _ => BASE_PRICE,
    })
    .into();

    // give user some funds
    let name_fee = coins(amount.u128(), NATIVE_DENOM);
    if Uint128::from(BASE_PRICE) > Uint128::from(0u128) {
        app.sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: user.to_string(),
                amount: name_fee.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
    }

    let msg = ExecuteMsg::MintAndList {
        account: name.to_string(),
    };

    app.execute_contract(
        Addr::unchecked(user),
        Addr::unchecked(MINTER),
        &msg,
        &name_fee,
    )
}

fn bid(app: &mut TestApp, name: &str, bidder: &str, amount: u128) {
    let bidder = Addr::unchecked(bidder);

    // give bidder some funds
    let amount = coins(amount, NATIVE_DENOM);
    app.sudo(CwSudoMsg::Bank({
        BankSudo::Mint {
            to_address: bidder.to_string(),
            amount: amount.clone(),
        }
    }))
    .map_err(|err| println!("{:?}", err))
    .ok();

    let msg = MarketplaceExecuteMsg::SetBid {
        token_id: name.to_string(),
    };
    let res = app.execute_contract(bidder.clone(), Addr::unchecked(MKT), &msg, &amount);
    assert!(res.is_ok());

    // query if bid exists
    let res: Option<Bid> = app
        .wrap()
        .query_wasm_smart(
            MKT,
            &(MarketplaceQueryMsg::Bid {
                token_id: name.to_string(),
                bidder: bidder.to_string(),
            }),
        )
        .unwrap();
    let bid = res.unwrap();
    assert_eq!(bid.token_id, name.to_string());
    assert_eq!(bid.bidder, bidder.to_string());
    assert_eq!(bid.amount, amount[0].amount);
}

mod execute {
    use bs721::{NftInfoResponse, OperatorsResponse};
    use bs721_account::QueryMsg as Bs721ProfileQueryMsg;
    use bs_std::NATIVE_DENOM;
    use btsg_account::market::state::{Ask, SudoParams};
    use btsg_account::Metadata;
    use cosmwasm_std::{attr, StdError};
    // use whitelist_updatable_flatrate::msg::QueryMsg::IncludesAddress;

    use crate::msg::QueryMsg;

    use super::*;

    #[test]
    fn check_approvals() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        // check operators
        let res: OperatorsResponse = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(bs721_base::msg::QueryMsg::<Empty>::AllOperators {
                    owner: USER.to_string(),
                    include_expired: None,
                    start_after: None,
                    limit: None,
                }),
            )
            .unwrap();
        assert_eq!(res.operators.len(), 1);
    }

    #[test]
    fn test_mint() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        // check if name is listed in marketplace
        let res: Option<Ask> = app
            .wrap()
            .query_wasm_smart(
                MKT,
                &(MarketplaceQueryMsg::Ask {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.unwrap().token_id, NAME);

        // check if token minted
        let _res: NumTokensResponse = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(COLLECTION),
                &(bs721_base::msg::QueryMsg::<Empty>::NumTokens {}),
            )
            .unwrap();

        assert_eq!(owner_of(&app, NAME.to_string()), USER.to_string());
    }

    #[test]
    fn test_bid() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());
        bid(&mut app, NAME, BIDDER, BID_AMOUNT);
    }

    #[test]
    fn test_accept_bid() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);

        // user (owner) starts off with 0 internet funny money
        let res = app
            .wrap()
            .query_balance(USER.to_string(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(res.amount, Uint128::new(0));

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(USER), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());

        // check if bid is removed
        let res: Option<Bid> = app
            .wrap()
            .query_wasm_smart(
                MKT,
                &(MarketplaceQueryMsg::Bid {
                    token_id: NAME.to_string(),
                    bidder: BIDDER.to_string(),
                }),
            )
            .unwrap();
        assert!(res.is_none());

        // verify that the bidder is the new owner
        assert_eq!(owner_of(&app, NAME.to_string()), BIDDER.to_string());

        // check if user got the bid amount
        let res = app
            .wrap()
            .query_balance(USER.to_string(), NATIVE_DENOM)
            .unwrap();
        let protocol_fee = 20_000_000u128;
        assert_eq!(res.amount, Uint128::from(BID_AMOUNT - protocol_fee));

        // confirm that a new ask was created
        let res: Option<Ask> = app
            .wrap()
            .query_wasm_smart(
                MKT,
                &(MarketplaceQueryMsg::Ask {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        let ask = res.unwrap();
        assert_eq!(ask.token_id, NAME);
        assert_eq!(ask.seller, BIDDER.to_string());
    }

    //  test two sales cycles in a row to check if approvals work
    #[test]
    fn test_two_sales_cycles() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(USER), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER2, BID_AMOUNT);

        // have to approve marketplace spend for bid acceptor (bidder)
        let msg: Bs721AccountExecuteMsg = Bs721AccountExecuteMsg::Approve {
            spender: MKT.to_string(),
            token_id: NAME.to_string(),
            expires: None,
        };
        let res = app.execute_contract(
            Addr::unchecked(BIDDER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER2.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(BIDDER), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());
    }

    #[test]
    fn test_reverse_map() {
        let mut app = instantiate_contracts(None, None, None);
        // needs to use valid address for querying addresses
        let user = "bitsong1hsk6jryyqjfhp5dhc55tc9jtckygx0epmnl9d9";

        let res = mint_and_list(&mut app, NAME, user);
        assert!(res.is_ok());

        // when no associated address, query should throw error
        let res: Result<String, cosmwasm_std::StdError> = app.wrap().query_wasm_smart(
            COLLECTION,
            &(Bs721AccountQueryMsg::AssociatedAddress {
                name: NAME.to_string(),
            }),
        );
        assert!(res.is_err());

        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(user.to_string()),
        };
        let res = app
            .execute_contract(
                Addr::unchecked(user),
                Addr::unchecked(COLLECTION),
                &msg,
                &[],
            )
            .unwrap();

        // query associated address should return user
        let res: String = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Bs721AccountQueryMsg::AssociatedAddress {
                    name: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res, user.to_string());

        // added to get around rate limiting
        update_block_time(&mut app, 60);

        // associate another
        let name2 = "exam";
        let res = mint_and_list(&mut app, name2, user);
        assert!(res.is_ok());

        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: name2.to_string(),
            address: Some(user.to_string()),
        };
        app.execute_contract(
            Addr::unchecked(user),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        )
        .unwrap();

        let res: String = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(COLLECTION),
                &(Bs721AccountQueryMsg::Account {
                    address: user.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res, name2.to_string());

        // prev token_id should reset token_uri to None
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(COLLECTION),
                &(Bs721ProfileQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.token_uri, None);

        // token uri should be user address
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(COLLECTION),
                &(Bs721ProfileQueryMsg::NftInfo {
                    token_id: name2.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.token_uri, Some(user.to_string()));

        // remove address
        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: None,
        };
        let res = app.execute_contract(
            Addr::unchecked(user),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // confirm removed from nft info
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                Addr::unchecked(COLLECTION),
                &(Bs721ProfileQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.token_uri, None);

        // remove address
        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: name2.to_string(),
            address: None,
        };
        let res = app.execute_contract(
            Addr::unchecked(user),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // confirm removed from reverse names map
        let res: Result<String, StdError> = app.wrap().query_wasm_smart(
            Addr::unchecked(COLLECTION),
            &(Bs721AccountQueryMsg::Account {
                address: user.to_string(),
            }),
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_reverse_map_contract_address() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, ADMIN2);
        assert!(res.is_ok());

        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(MINTER.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(ADMIN2),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        res.unwrap().events.iter().for_each(|e| {
            if e.ty == "wasm-associate-address" {
                assert_eq!(e.attributes[1], attr("name", NAME));
                assert_eq!(e.attributes[2], attr("owner", ADMIN2));
                assert_eq!(e.attributes[3], attr("address", MINTER));
            }
        });
    }

    #[test]
    fn test_reverse_map_not_contract_address_admin() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, ADMIN2);
        assert!(res.is_ok());

        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(MINTER.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_reverse_map_not_owner() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(USER2.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER2),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_pause() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);

        // verify addr in wl
        // let whitelists: Vec<Addr> = app
        //     .wrap()
        //     .query_wasm_smart(MINTER, &(QueryMsg::Whitelists {}))
        //     .unwrap();

        // assert_eq!(whitelists.len(), 1);

        // whitelists.iter().find(|whitelist| {
        //     let included: bool = app
        //         .wrap()
        //         .query_wasm_smart(
        //             Addr::unchecked(whitelist.to_string()),
        //             &(IncludesAddress {
        //                 address: USER.to_string(),
        //             }),
        //         )
        //         .unwrap();
        //     dbg!(included, whitelist);
        //     included
        // });

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        let msg = ExecuteMsg::Pause { pause: true };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        let err = mint_and_list(&mut app, "name2", USER);
        assert!(err.is_err());
    }

    #[test]
    fn update_mkt_sudo() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        let msg = MarketplaceSudoMsg::UpdateParams {
            trading_fee_bps: Some(1000u64),
            min_price: Some(Uint128::from(1000u128)),
            ask_interval: Some(1000),
        };

        let res = app.wasm_sudo(Addr::unchecked(MKT), &msg);
        assert!(res.is_ok());

        let res: SudoParams = app
            .wrap()
            .query_wasm_smart(Addr::unchecked(MKT), &(QueryMsg::Params {}))
            .unwrap();
        let params = res;

        assert_eq!(params.trading_fee_percent, Decimal::percent(10));
        assert_eq!(params.min_price, Uint128::from(1000u128));
        assert_eq!(params.ask_interval, 1000);
    }
}

mod admin {

    use super::*;

    #[test]
    fn update_admin() {
        let mut app = instantiate_contracts(None, Some(ADMIN.to_string()), None);

        let msg = ExecuteMsg::UpdateAdmin { admin: None };
        let res = app.execute_contract(Addr::unchecked(USER), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_err());

        let msg = ExecuteMsg::UpdateAdmin {
            admin: Some(USER2.to_string()),
        };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        let msg = ExecuteMsg::UpdateAdmin { admin: None };
        let res = app.execute_contract(Addr::unchecked(USER2), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        // cannot update admin after its been removed
        let msg = ExecuteMsg::UpdateAdmin { admin: None };
        let res = app.execute_contract(Addr::unchecked(USER2), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_err());
    }
}

mod query {
    use bs721_base::msg::QueryMsg as Bs721QueryMsg;
    use btsg_account::market::{state::Ask, BidOffset};
    use cosmwasm_std::StdResult;

    use super::*;

    #[test]
    fn query_ask() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: Option<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.unwrap().token_id, NAME.to_string());
    }

    #[test]
    fn query_asks() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER).unwrap();

        let res = mint_and_list(&mut app, "hack", ADMIN2);
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::Asks {
            start_after: None,
            limit: None,
        };
        let res: Vec<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res[0].id, 1);
    }

    #[test]
    fn query_asks_by_seller() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        let res = mint_and_list(&mut app, "hack", USER2);
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::AsksBySeller {
            seller: USER.to_string(),
            start_after: None,
            limit: None,
        };
        let res: Vec<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn query_ask_count() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        let res = mint_and_list(&mut app, "hack", ADMIN2);
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::AskCount {};
        let res: u64 = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res, 2);
    }

    #[test]
    fn query_top_bids() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);
        bid(&mut app, NAME, BIDDER2, BID_AMOUNT * 5);

        let msg = MarketplaceQueryMsg::ReverseBidsSortedByPrice {
            start_before: None,
            limit: None,
        };
        let res: Vec<Bid> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].amount.u128(), BID_AMOUNT * 5);
    }

    #[test]
    fn query_bids_by_seller() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);
        bid(&mut app, NAME, BIDDER2, BID_AMOUNT * 5);

        let msg = MarketplaceQueryMsg::BidsForSeller {
            seller: USER.to_string(),
            start_after: None,
            limit: None,
        };
        let res: Vec<Bid> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].amount.u128(), BID_AMOUNT);

        // test pagination
        let bid_offset = BidOffset {
            price: Uint128::from(BID_AMOUNT),
            bidder: Addr::unchecked(BIDDER),
            token_id: NAME.to_string(),
        };
        let msg = MarketplaceQueryMsg::BidsForSeller {
            seller: USER.to_string(),
            start_after: Some(bid_offset),
            limit: None,
        };
        let res: Vec<Bid> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        // should be length 0 because there are no token_ids besides NAME.to_string()
        assert_eq!(res.len(), 0);

        // added to get around rate limiting
        update_block_time(&mut app, 60);

        // test pagination with multiple names and bids
        let name = "jump";
        let res = mint_and_list(&mut app, name, USER);
        assert!(res.is_ok());
        bid(&mut app, name, BIDDER, BID_AMOUNT * 3);
        bid(&mut app, name, BIDDER2, BID_AMOUNT * 2);
        let res: Vec<Bid> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        // should be length 2 because there is token_id "jump" with 2 bids
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn query_highest_bid() -> anyhow::Result<()> {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER)?;
        // assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);
        bid(&mut app, NAME, BIDDER2, BID_AMOUNT * 5);

        let msg = MarketplaceQueryMsg::HighestBid {
            token_id: NAME.to_string(),
        };
        let res: Option<Bid> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.unwrap().amount.u128(), BID_AMOUNT * 5);
        Ok(())
    }

    #[test]
    fn query_name() {
        let mut app = instantiate_contracts(None, None, None);
        mint_and_list(&mut app, NAME, USER).unwrap();

        // fails with "user" string, has to be a bech32 address
        let res: StdResult<String> = app.wrap().query_wasm_smart(
            COLLECTION,
            &(Bs721AccountQueryMsg::Account {
                address: USER2.to_string(),
            }),
        );
        assert!(res.is_err());

        let user = "bitsong1hsk6jryyqjfhp5dhc55tc9jtckygx0epmnl9d9";

        mint_and_list(&mut app, "yoyo", USER).unwrap();

        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: "yoyo".to_string(),
            address: Some(user.to_string()),
        };
        let res = app
            .execute_contract(
                Addr::unchecked(USER),
                Addr::unchecked(COLLECTION),
                &msg,
                &[],
            )
            .unwrap();

        let res: String = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Bs721AccountQueryMsg::Account {
                    address: user.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res, "yoyo".to_string());
    }

    #[test]
    fn query_trading_start_time() {
        let app = instantiate_contracts(None, None, None);

        // let res: CollectionInfoResponse = app
        //     .wrap()
        //     .query_wasm_smart(COLLECTION, &(Bs721QueryMsg::<Empty>::CollectionInfo {}))
        //     .unwrap();
        // assert_eq!(
        //     res.start_trading_time.unwrap(),
        //     app.block_info()
        //         .time
        //         .plus_seconds(TRADING_START_TIME_OFFSET_IN_SECONDS)
        // );
    }
}

mod collection {
    use bs721::NftInfoResponse;
    use bs721_account::msg::Bs721AccountsQueryMsg;
    use btsg_account::{
        market::state::Ask,
        {Metadata, TextRecord, NFT},
    };
    use cosmwasm_std::{to_json_binary, StdResult};
    use cw_controllers::AdminResponse;

    use super::*;

    pub(crate) fn transfer(app: &mut TestApp, from: &str, to: &str) {
        let msg = Bs721AccountExecuteMsg::TransferNft {
            recipient: to.to_string(),
            token_id: NAME.to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(from),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: Option<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.unwrap().seller.to_string(), to.to_string());
    }

    fn send(app: &mut TestApp, from: &str, to: &str) {
        let msg = to_json_binary("You now have the melting power").unwrap();
        let target = to.to_string();
        let send_msg = Bs721AccountExecuteMsg::SendNft {
            contract: target,
            token_id: NAME.to_string(),
            msg,
        };
        let res = app.execute_contract(
            Addr::unchecked(from),
            Addr::unchecked(COLLECTION),
            &send_msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: Option<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.unwrap().seller.to_string(), to.to_string());
    }

    #[test]
    fn verify_twitter() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        let name = "twitter";
        let value = "shan3v";

        let msg = Bs721AccountExecuteMsg::AddTextRecord {
            name: NAME.to_string(),
            record: TextRecord::new(name, value),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // query text record to see if verified is not set
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Bs721AccountsQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        let msg = Bs721AccountExecuteMsg::VerifyTextRecord {
            name: NAME.to_string(),
            record_name: name.to_string(),
            result: true,
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        // fails cuz caller is not oracle verifier
        assert!(res.is_err());

        let res = app.execute_contract(
            Addr::unchecked(VERIFIER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = Bs721AccountsQueryMsg::Verifier {};
        let verifier: AdminResponse = app.wrap().query_wasm_smart(COLLECTION, &msg).unwrap();
        assert_eq!(verifier.admin, Some(VERIFIER.to_string()));

        // query text record to see if verified is set
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Bs721AccountsQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, Some(true));
    }

    #[test]
    fn verify_false() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        let name = "twitter";
        let value = "shan3v";

        let msg = Bs721AccountExecuteMsg::AddTextRecord {
            name: NAME.to_string(),
            record: TextRecord::new(name, value),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // verify something as false
        let msg = Bs721AccountExecuteMsg::VerifyTextRecord {
            name: NAME.to_string(),
            record_name: name.to_string(),
            result: false,
        };
        let res = app.execute_contract(
            Addr::unchecked(VERIFIER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // query text record to see if verified is set
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Bs721AccountsQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, Some(false));
    }

    #[test]
    fn verified_text_record() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        let name = "twitter";
        let value = "shan3v";

        let msg = Bs721AccountExecuteMsg::AddTextRecord {
            name: NAME.to_string(),
            record: TextRecord {
                name: name.to_string(),
                value: value.to_string(),
                verified: Some(true),
            },
        };

        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // query text record to see if verified is not set
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(bs721::Bs721QueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        // attempt update text record w verified value
        let msg = Bs721AccountExecuteMsg::UpdateTextRecord {
            name: NAME.to_string(),
            record: TextRecord {
                name: name.to_string(),
                value: "some new value".to_string(),
                verified: Some(true),
            },
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // query text record to see if verified is not set
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Bs721AccountsQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        // query text record to see if verified is not set
        let res: NftInfoResponse<Metadata> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Bs721AccountsQueryMsg::NftInfo {
                    token_id: NAME.to_string(),
                }),
            )
            .unwrap();
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        // query image nft
        let res: Option<NFT> = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &(Bs721AccountsQueryMsg::ImageNFT {
                    name: NAME.to_string(),
                }),
            )
            .unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn transfer_nft() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        transfer(&mut app, USER, USER2);
    }

    #[test]
    fn send_nft() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        send(&mut app, USER, USER2);
    }

    #[test]
    fn transfer_nft_and_bid() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        transfer(&mut app, USER, USER2);

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);

        // user2 must approve the marketplace to transfer their name
        let msg = Bs721AccountExecuteMsg::Approve {
            spender: MKT.to_string(),
            token_id: NAME.to_string(),
            expires: None,
        };
        let res = app.execute_contract(
            Addr::unchecked(USER2),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // accept bid
        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(USER2), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());
    }

    #[test]
    fn transfer_nft_with_reverse_map() {
        let mut app = instantiate_contracts(None, None, None);

        let user = "bitsong1hsk6jryyqjfhp5dhc55tc9jtckygx0epmnl9d9";
        let user2 = "bitsong1wh3wjjgprxeww4cgqyaw8k75uslzh3sdf903qg";
        let res = mint_and_list(&mut app, NAME, user);
        assert!(res.is_ok());

        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(user.to_string()),
        };
        let res = app
            .execute_contract(
                Addr::unchecked(user),
                Addr::unchecked(COLLECTION),
                &msg,
                &[],
            )
            .unwrap();

        let msg = Bs721AccountQueryMsg::Account {
            address: user.to_string(),
        };
        let res: String = app.wrap().query_wasm_smart(COLLECTION, &msg).unwrap();
        assert_eq!(res, NAME.to_string());

        transfer(&mut app, user, user2);

        let msg = Bs721AccountQueryMsg::Account {
            address: user.to_string(),
        };
        let err: StdResult<String> = app.wrap().query_wasm_smart(COLLECTION, &msg);
        assert!(err.is_err());

        let msg = Bs721AccountQueryMsg::Account {
            address: user2.to_string(),
        };
        let err: StdResult<String> = app.wrap().query_wasm_smart(COLLECTION, &msg);
        assert!(err.is_err());
    }

    // test that burn nft currently does nothing. this is a placeholder for future functionality
    #[test]
    fn burn_nft() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        let msg = Bs721AccountExecuteMsg::Burn {
            token_id: NAME.to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_err());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: Option<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert!(res.is_some());
    }

    // test that burn nft currently does nothing. this is a placeholder for future functionality
    #[test]
    fn burn_with_existing_bids() {
        let mut app = instantiate_contracts(None, None, None);

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        bid(&mut app, NAME, BIDDER, BID_AMOUNT);

        let msg = Bs721AccountExecuteMsg::Burn {
            token_id: NAME.to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_err());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: Option<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        let ask = res.unwrap();
        assert_eq!(ask.seller.to_string(), USER.to_string());
    }

    // test that burn nft currently does nothing. this is a placeholder for future functionality
    #[test]
    fn burn_nft_with_reverse_map() {
        let mut app = instantiate_contracts(None, None, None);

        let user = "bitsong1hsk6jryyqjfhp5dhc55tc9jtckygx0epmnl9d9";

        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_ok());

        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(user.to_string()),
        };
        app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        )
        .unwrap();

        let msg = Bs721AccountQueryMsg::Account {
            address: user.to_string(),
        };
        let res: String = app.wrap().query_wasm_smart(COLLECTION, &msg).unwrap();
        assert_eq!(res, NAME.to_string());

        let msg = Bs721AccountExecuteMsg::Burn {
            token_id: NAME.to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(user),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_err());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: Option<Ask> = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert!(res.is_some());

        let msg = Bs721AccountQueryMsg::Account {
            address: user.to_string(),
        };
        let res: StdResult<String> = app.wrap().query_wasm_smart(COLLECTION, &msg);
        assert!(res.is_ok());
    }

    #[test]
    fn sudo_update() {
        let mut app = instantiate_contracts(None, None, None);
        let params: Bs721SudoParams = app
            .wrap()
            .query_wasm_smart(COLLECTION, &(Bs721AccountsQueryMsg::Params {}))
            .unwrap();
        let max_record_count = params.max_record_count;

        let msg = bs721_account::msg::SudoMsg::UpdateParams {
            max_record_count: max_record_count + 1,
        };
        let res = app.wasm_sudo(Addr::unchecked(COLLECTION), &msg);
        assert!(res.is_ok());
        let params: Bs721SudoParams = app
            .wrap()
            .query_wasm_smart(COLLECTION, &(Bs721AccountsQueryMsg::Params {}))
            .unwrap();
        assert_eq!(params.max_record_count, max_record_count + 1);
    }
}

mod public_start_time {
    use btsg_account::minter::Config;

    use crate::msg::QueryMsg;

    use super::*;

    #[test]
    fn mint_before_start() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)),
        );

        let res = mint_and_list(&mut app, NAME, USER2);
        assert!(res.is_err());

        // try pub mint before start time
        let res = mint_and_list(&mut app, NAME, USER);
        assert!(res.is_err());
    }

    #[test]
    fn update_start_time() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)),
        );

        // default start time is PUBLIC_MINT_START_TIME_IN_SECONDS
        let msg = QueryMsg::Config {};
        let res: Config = app.wrap().query_wasm_smart(MINTER, &msg).unwrap();
        assert_eq!(
            res.public_mint_start_time,
            PUBLIC_MINT_START_TIME_IN_SECONDS
        );

        // update start time to PUBLIC_MINT_START_TIME_IN_SECONDS - 1
        let msg = ExecuteMsg::UpdateConfig {
            config: Config {
                public_mint_start_time: PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1),
            },
        };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        // check start time
        let msg = QueryMsg::Config {};
        let res: Config = app.wrap().query_wasm_smart(MINTER, &msg).unwrap();
        assert_eq!(
            res.public_mint_start_time,
            PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)
        );

        // mint succeeds w new mint start time
        let res = mint_and_list(&mut app, NAME, USER).unwrap();
    }
}

mod associate_address {
    use super::*;

    use collection::transfer;

    use bs721_base::InstantiateMsg as Bs721InstantiateMsg;
    use cosmwasm_std::BlockInfo;
    use cw_multi_test::next_block;

    #[test]
    fn transfer_to_eoa() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)),
        );

        let nft_id = app.store_code(contract_nft());

        let init_msg = Bs721InstantiateMsg {
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            minter: Addr::unchecked(MINTER).to_string(),
            uri: None,
        };
        let nft_addr = app
            .instantiate_contract(
                nft_id,
                Addr::unchecked(MINTER),
                &init_msg,
                &[],
                "NFT",
                Some(ADMIN.to_string()),
            )
            .unwrap();
        let BlockInfo {
            height,
            time,
            chain_id,
        } = app.block_info();
        app.set_block(BlockInfo {
            height: height.checked_add(1).unwrap(),
            time: time.plus_seconds(2),
            chain_id,
        });
        // mint and transfer to collection
        mint_and_list(&mut app, NAME, USER).unwrap();
        transfer(&mut app, USER, nft_addr.as_ref());
        let owner = owner_of(&app, NAME.to_string());
        assert_eq!(owner, nft_addr.to_string());

        // associate contract
        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(nft_addr.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(nft_addr.clone()),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // transfer from collection back to personal wallet
        transfer(&mut app, nft_addr.as_ref(), USER);
        let owner = owner_of(&app, NAME.to_string());
        assert_eq!(owner, USER.to_string());
    }

    #[test]
    fn associate_with_a_contract_with_no_admin() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)),
        );

        let nft_id = app.store_code(contract_nft());

        // For the purposes of this test, a collection contract with no admin needs to be instantiated (contract_with_no_admin)
        // This contract needs to have a creator that is itself a contract and this creator contract should have an admin (USER).
        // The admin (USER) of the creator contract will mint a name and associate the name with the collection contract that doesn't have an admin successfully.

        // Instantiating the creator contract with an admin (USER)
        let creator_init_msg = Bs721InstantiateMsg {
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            minter: Addr::unchecked(MINTER).to_string(),
            uri: None,
        };
        let creator_addr = app
            .instantiate_contract(
                nft_id,
                Addr::unchecked(MINTER),
                &creator_init_msg,
                &[],
                "NFT",
                Some(USER.to_string()),
            )
            .unwrap();

        let mut block = app.block_info();
        block.time = block.time.plus_seconds(1);
        app.set_block(block);

        // The creator contract instantiates the collection contract with no admin
        let init_msg = Bs721InstantiateMsg {
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            minter: creator_addr.to_string(),
            uri: None,
        };

        let collection_with_no_admin_addr = app
            .instantiate_contract(nft_id, creator_addr, &init_msg, &[], "NFT", None)
            .unwrap();

        // USER mints a name
        mint_and_list(&mut app, NAME, USER).unwrap();

        // USER associates the name with the collection contract that doesn't have an admin
        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(collection_with_no_admin_addr.to_string()),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());
    }
    #[test]
    fn associate_with_a_contract_with_no_admin_fail() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)),
        );

        let nft_id = app.store_code(contract_nft());

        // For the purposes of this test, a collection contract with no admin needs to be instantiated (contract_with_no_admin)
        // This contract needs to have a creator that is itself a contract and this creator contract should have an admin (USER).
        // An address other than the admin (USER) of the creator contract will mint a name, try to associate the name with the collection contract that doesn't have an admin and fail.

        // Instantiating the creator contract with an admin (USER)
        let creator_init_msg = Bs721InstantiateMsg {
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            minter: Addr::unchecked(MINTER).to_string(),
            uri: None,
        };
        let creator_addr = app
            .instantiate_contract(
                nft_id,
                Addr::unchecked(MINTER),
                &creator_init_msg,
                &[],
                "NFT",
                Some(USER.to_string()),
            )
            .unwrap();

        // The creator contract instantiates the collection contract with no admin
        let init_msg = Bs721InstantiateMsg {
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            minter: creator_addr.to_string(),
            uri: None,
        };

        let collection_with_no_admin_addr = app
            .instantiate_contract(nft_id, creator_addr, &init_msg, &[], "NFT", None)
            .unwrap();

        let mut block = app.block_info();
        block.time = block.time.plus_seconds(1);
        app.set_block(block);

        // USER4 mints a name
        mint_and_list(&mut app, NAME, USER4).unwrap();

        // USER4 tries to associate the name with the collection contract that doesn't have an admin
        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(collection_with_no_admin_addr.to_string()),
        };
        let res = app
            .execute_contract(
                Addr::unchecked(USER4),
                Addr::unchecked(COLLECTION),
                &msg,
                &[],
            )
            .map_err(|e| e.downcast::<bs721_account::ContractError>().unwrap())
            .unwrap_err();
        assert!(matches!(
            res,
            bs721_account::ContractError::UnauthorizedCreatorOrAdmin {}
        ))
    }

    #[test]
    fn associate_with_a_contract_with_an_admin_fail() {
        let mut app = instantiate_contracts(
            None,
            Some(ADMIN.to_string()),
            Some(PUBLIC_MINT_START_TIME_IN_SECONDS.minus_seconds(1)),
        );

        let nft_id = app.store_code(contract_nft());

        // Instantiating the creator contract with an admin (USER)
        let creator_init_msg = Bs721InstantiateMsg {
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            minter: Addr::unchecked(MINTER).to_string(),
            uri: None,
        };
        let contract_with_an_admin = app
            .instantiate_contract(
                nft_id,
                Addr::unchecked(MINTER),
                &creator_init_msg,
                &[],
                "NFT",
                Some(USER.to_string()),
            )
            .unwrap();

        app.set_block(BlockInfo {
            height: app.block_info().height,
            time: app.block_info().time.plus_seconds(2u64),
            chain_id: app.block_info().chain_id,
        });

        // USER4 mints a name
        mint_and_list(&mut app, NAME, USER4).unwrap();

        // USER4 tries to associate the name with the collection contract that has an admin (USER)
        let msg = Bs721AccountExecuteMsg::AssociateAddress {
            name: NAME.to_string(),
            address: Some(contract_with_an_admin.to_string()),
        };
        let res = app
            .execute_contract(
                Addr::unchecked(USER4),
                Addr::unchecked(COLLECTION),
                &msg,
                &[],
            )
            .map_err(|e| e.downcast::<bs721_account::ContractError>().unwrap())
            .unwrap_err();
        assert!(matches!(
            res,
            bs721_account::ContractError::UnauthorizedCreatorOrAdmin {}
        ))
    }
}
