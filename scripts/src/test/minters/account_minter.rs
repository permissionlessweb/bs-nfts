use btsg_account::common::SECONDS_PER_YEAR;
use cw_orch::{anyhow, mock::MockBech32, prelude::*};

use crate::bundles::account::BtsgAccountSuite;
use btsg_account::market::{ExecuteMsgFns as _, QueryMsgFns, SudoMsg as MarketplaceSudoMsg};
use cosmwasm_std::{attr, coins, to_json_binary, Decimal};
use cw_orch::mock::cw_multi_test::{SudoMsg, WasmSudo};

use bs721_account_minter::msg::QueryMsgFns as _;
use btsg_account::minter::Config;

use bs721_account_minter::msg::ExecuteMsgFns as _;
use cosmwasm_std::Uint128;

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
const TRADING_START_TIME_OFFSET_IN_SECONDS: u64 = 2 * SECONDS_PER_YEAR;
const TRADING_FEE_BPS: u64 = 200; // 2%

const BID_AMOUNT: u128 = 1_000_000_000;
const PER_ADDRESS_LIMIT: u32 = 2;

const MKT: &str = "contract0";
const MINTER: &str = "contract1";
const COLLECTION: &str = "contract2";
const WHITELIST: &str = "contract3";

#[test]
pub fn init() -> anyhow::Result<()> {
    // new mock Bech32 chain environment
    let mock = MockBech32::new("mock");
    // simulate deploying the test suite to the mock chain env.
    BtsgAccountSuite::deploy_on(mock.clone(), mock.sender)?;
    Ok(())
}

mod execute {

    use btsg_account::account::{Bs721AccountsQueryMsgFns, ExecuteMsgFns};

    use super::*;

    #[test]
    fn test_check_approvals() -> anyhow::Result<()> {
        let mock = MockBech32::new("mock");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;

        let owner = mock.sender.clone();
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &owner)?;
        // check operators
        assert_eq!(
            suite
                .account
                .all_operators(owner, None, None, None)?
                .operators
                .len(),
            1
        );

        Ok(())
    }
    #[test]
    fn test_mint() -> anyhow::Result<()> {
        let mock = MockBech32::new("mock");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        let owner = mock.sender.clone();
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &owner)?;

        // check if name is listed in marketplace
        let res = suite.market.ask(token_id.to_string())?.unwrap();
        assert_eq!(res.token_id, token_id);

        // check if token minted
        let res = suite.account.num_tokens()?;
        assert_eq!(res.count, 1);

        assert_eq!(suite.owner_of(token_id.into())?, owner.to_string());

        Ok(())
    }
    #[test]
    fn test_bid() -> anyhow::Result<()> {
        let mock = MockBech32::new("mock");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        let owner = mock.sender.clone();
        let bidder = mock.addr_make("bidder");
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &owner)?;

        suite.bid_w_funds(mock, &token_id, bidder, BID_AMOUNT)?;
        Ok(())
    }
    #[test]
    fn test_accept_bid() -> anyhow::Result<()> {
        let mock = MockBech32::new("mock");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        let owner = mock.sender.clone();
        let bidder = mock.addr_make("bidder");
        let token_id = "bobo";

        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &owner)?;
        suite.bid_w_funds(mock.clone(), &token_id, bidder.clone(), BID_AMOUNT)?;

        // user (owner) starts off with 0 internet funny money
        assert_eq!(
            mock.balance(owner.clone(), Some("ubtsg".into()))?[0].amount,
            Uint128::zero()
        );

        suite.market.accept_bid(bidder.clone(), token_id.into())?;

        // check if bid is removed
        assert!(suite
            .market
            .bid(bidder.to_string(), token_id.into())?
            .is_none());
        // verify that the bidder is the new owner
        assert_eq!(suite.owner_of(token_id.into())?, bidder.to_string());
        // check if user got the bid amount
        assert_eq!(
            mock.balance(owner, Some("ubtsg".into()))?[0].amount,
            Uint128::from(BID_AMOUNT)
        );
        // confirm that a new ask was created
        let res = suite.market.ask(token_id.to_string())?.unwrap();
        assert_eq!(res.seller, bidder);
        assert_eq!(res.token_id, token_id);
        Ok(())
    }
    #[test]

    fn test_two_sales_cycles() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        let owner = mock.sender.clone();
        let bidder = mock.addr_make("bidder");
        let bidder2 = mock.addr_make("bidder2");
        let token_id = "bobo";
        let market = suite.market.address()?;
        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &owner)?;
        suite.bid_w_funds(mock.clone(), &token_id, bidder.clone(), BID_AMOUNT)?;
        suite.market.accept_bid(bidder.clone(), token_id.into())?;
        suite.bid_w_funds(mock.clone(), &token_id, bidder2.clone(), BID_AMOUNT)?;
        suite
            .account
            .call_as(&bidder)
            .approve(market, token_id, None)?;

        suite
            .market
            .call_as(&bidder)
            .accept_bid(bidder2.clone(), token_id.into())?;

        Ok(())
    }
    #[test]
    fn test_reverse_map() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        mock.wait_seconds(1)?;

        let admin2 = mock.addr_make("admin2");
        let token_id = "bobo";

        suite.mint_and_list(mock.clone(), &token_id, &admin2)?;

        // when no associated address, query should throw error
        suite
            .account
            .call_as(&admin2)
            .associated_address(token_id)
            .unwrap_err();

        // associate owner address with account name
        suite
            .account
            .call_as(&admin2)
            .associate_address(token_id, Some(admin2.to_string()))?;

        // query associated address should return user
        assert_eq!(suite.account.associated_address(token_id)?, admin2);

        // added to get around rate limiting
        mock.wait_seconds(60)?;
        // associate another
        let account2 = "exam";
        suite.mint_and_list(mock.clone(), &account2, &admin2.clone())?;

        suite
            .account
            .call_as(&admin2)
            .associate_address(account2, Some(admin2.to_string()))?;

        assert_eq!(suite.account.account(admin2)?, account2.to_string());
        Ok(())
    }

    #[test]
    fn test_reverse_map_contract_address() -> anyhow::Result<()> {
        Ok(())
    }
    #[test]
    fn test_reverse_map_not_contract_address_admin() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;

        let addr = mock.addr_make_with_balance("not-admin", coins(1000000000, "ubtsg"))?;
        let minter = suite.minter.address()?;
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &addr)?;

        suite
            .account
            .associate_address(token_id, Some(minter.to_string()))
            .unwrap_err();
        Ok(())
    }
    #[test]
    fn test_reverse_map_not_owner() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        let token_id = "bobo";
        let admin2 = mock.addr_make("admin2");
        let admin = mock.sender.clone();

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &admin2)?;

        suite
            .account
            .associate_address(token_id, Some(admin2.to_string()))
            .unwrap_err();
        Ok(())
    }
    #[test]
    fn test_pause() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let token_id = "bobo";
        let admin2 = mock.addr_make("admin2");

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &admin2)?;

        // pause minting
        suite.minter.pause(true)?;

        // error trying to mint
        mock.wait_seconds(1)?;
        suite
            .mint_and_list(mock.clone(), &token_id, &admin2)
            .unwrap_err();

        Ok(())
    }
    #[test]
    fn test_update_mkt_sudo() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, None)?;
        let token_id = "bobo";
        let admin2 = mock.addr_make("admin2");

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &admin2)?;

        // run sudo msg
        mock.app.borrow_mut().sudo(SudoMsg::Wasm(WasmSudo {
            contract_addr: suite.market.address()?,
            message: to_json_binary(&MarketplaceSudoMsg::UpdateParams {
                trading_fee_bps: Some(1000u64),
                min_price: Some(Uint128::from(1000u128)),
                ask_interval: Some(1000),
            })?,
        }))?;

        // confirm updated params
        let res = suite.market.params()?;
        assert_eq!(res.trading_fee_percent, Decimal::percent(10));
        assert_eq!(res.min_price, Uint128::from(1000u128));
        assert_eq!(res.ask_interval, 1000);

        Ok(())
    }
}
mod admin {
    use super::*;

    #[test]
    fn test_update_admin() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin2 = mock.addr_make("admin2");
        // non-admin tries to set admin to None
        suite
            .minter
            .call_as(&admin2)
            .update_admin(None)
            .unwrap_err();
        // admin updates admin
        suite.minter.update_admin(Some(admin2.to_string()))?;
        // new admin updates to have no admin
        suite.minter.call_as(&admin2).update_admin(None)?;
        // cannot update without admin
        suite
            .minter
            .update_admin(Some(admin2.to_string()))
            .unwrap_err();
        Ok(())
    }
}
mod query {
    use btsg_account::{
        account::{Bs721AccountsQueryMsgFns, ExecuteMsgFns},
        market::BidOffset,
    };

    use super::*;

    #[test]
    fn test_query_ask() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin2 = mock.addr_make("admin2");
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &admin2)?;

        assert_eq!(
            suite.market.ask(token_id.into())?.unwrap().token_id,
            token_id
        );
        Ok(())
    }
    #[test]
    fn test_query_asks() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let admin2 = mock.addr_make("admin2");
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &admin)?;
        suite.mint_and_list(mock.clone(), &"hack", &admin2)?;

        assert_eq!(suite.market.asks(None, None)?[0].id, 1);

        Ok(())
    }
    #[test]
    fn test_query_asks_by_seller() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let admin2 = mock.addr_make("admin2");
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &admin)?;
        suite.mint_and_list(mock.clone(), &"hack", &admin2)?;

        assert_eq!(
            suite
                .market
                .asks_by_seller(admin.to_string(), None, None)?
                .len(),
            1
        );
        Ok(())
    }
    #[test]
    fn test_query_ask_count() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let admin2 = mock.addr_make("admin2");
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &admin)?;
        suite.mint_and_list(mock.clone(), &"hack", &admin2)?;

        assert_eq!(suite.market.ask_count()?, 2);
        Ok(())
    }
    #[test]
    fn test_query_top_bids() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let admin2 = mock.addr_make("admin2");
        let bidder1 = mock.addr_make("bidder1");
        let bidder2 = mock.addr_make("bidder2");
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &admin)?;

        suite.bid_w_funds(mock.clone(), &token_id, bidder1.clone(), BID_AMOUNT)?;
        suite.bid_w_funds(mock.clone(), &token_id, bidder2.clone(), BID_AMOUNT * 5)?;

        let res = suite.market.bids_for_seller(admin.clone(), None, None)?;
        assert_eq!(res.len(), 2);
        assert_eq!(res[1].amount.u128(), BID_AMOUNT);

        // test pagination
        let filter = BidOffset {
            price: Uint128::from(BID_AMOUNT),
            token_id: token_id.into(),
            bidder: bidder1.clone(),
        };
        let res: Vec<btsg_account::market::state::Bid> =
            suite
                .market
                .bids_for_seller(admin.clone(), None, Some(filter.clone()))?;

        // should be length 0 because there are no token_ids besides NAME.to_string()
        assert_eq!(res.len(), 0);

        // added to get around rate limiting
        mock.wait_seconds(60)?;

        // test pagination with multiple names and bids
        let name = "jump";
        let res = suite.mint_and_list(mock.clone(), name, &admin)?;
        suite.bid_w_funds(mock.clone(), name, bidder1, BID_AMOUNT * 3)?;
        suite.bid_w_funds(mock.clone(), name, bidder2, BID_AMOUNT * 2)?;
        let res = suite.market.bids_for_seller(admin, None, Some(filter))?;
        // should be length 2 because there is token_id "jump" with 2 bids
        assert_eq!(res.len(), 2);

        Ok(())
    }
    #[test]
    fn test_query_bids_by_seller() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let admin2 = mock.addr_make("admin2");
        let bidder1 = mock.addr_make("bidder1");
        let bidder2 = mock.addr_make("bidder2");
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &admin)?;

        suite.bid_w_funds(mock.clone(), &token_id, bidder1.clone(), BID_AMOUNT)?;
        suite.bid_w_funds(mock.clone(), &token_id, bidder2.clone(), BID_AMOUNT * 5)?;

        let res = suite.market.bids_for_seller(admin.clone(), None, None)?;
        assert_eq!(res.len(), 2);
        assert_eq!(res[1].amount.u128(), BID_AMOUNT);

        // test pagination
        let res = suite.market.bids_for_seller(
            admin.clone(),
            None,
            Some(BidOffset::new(
                Uint128::from(BID_AMOUNT),
                token_id.to_string(),
                bidder1.clone(),
            )),
        )?;
        assert_eq!(res.len(), 0);

        // added to get around rate limiting
        mock.wait_seconds(60)?;

        let name = "jump";
        suite.mint_and_list(mock.clone(), &name, &admin)?;
        suite.bid_w_funds(mock.clone(), name, bidder1.clone(), BID_AMOUNT * 3)?;
        suite.bid_w_funds(mock.clone(), name, bidder2, BID_AMOUNT * 2)?;
        // should be length 2 because there is token_id "jump" with 2 bids
        let res = suite.market.bids_for_seller(
            admin.clone(),
            None,
            Some(BidOffset::new(
                Uint128::from(BID_AMOUNT),
                token_id.to_string(),
                bidder1.clone(),
            )),
        )?;
        assert_eq!(res.len(), 2);

        Ok(())
    }
    #[test]
    fn test_query_highest_bid() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let bidder1 = mock.addr_make("bidder1");
        let bidder2 = mock.addr_make("bidder2");
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &admin)?;

        suite.bid_w_funds(mock.clone(), &token_id, bidder1.clone(), BID_AMOUNT)?;
        suite.bid_w_funds(mock.clone(), &token_id, bidder2.clone(), BID_AMOUNT * 5)?;

        assert_eq!(
            suite
                .market
                .highest_bid(token_id.to_string())?
                .unwrap()
                .amount
                .u128(),
            BID_AMOUNT * 5
        );

        Ok(())
    }
    #[test]
    fn test_query_name() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let admin = mock.sender.clone();
        let user1 = mock.addr_make("user1");
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &admin)?;

        // fails with "user" string, has to be a bech32 address
        suite.account.account(token_id).unwrap_err();

        suite.mint_and_list(mock.clone(), &"yoyo", &admin)?;

        suite
            .account
            .associate_address("yoyo", Some(admin.to_string()))?;

        assert_eq!(suite.account.account(admin)?, "yoyo".to_string());

        Ok(())
    }
    // #[test]
    // fn test_query_trading_start_time() -> anyhow::Result<()> {
    //     let mock = MockBech32::new("bitsong");
    //     let mut suite = BtsgAccountSuite::new(mock.clone());
    //     suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

    //     Ok(())
    // }
}
mod collection {
    use btsg_account::{
        account::{Bs721AccountsQueryMsgFns, ExecuteMsgFns},
        TextRecord,
    };

    use super::*;
    #[test]
    fn test_verify_twitter() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        mock.wait_seconds(1)?;

        let admin_user = mock.sender.clone();
        let verifier = mock.addr_make("verifier");
        let token_id = "bobo";

        suite.mint_and_list(mock, token_id, &admin_user)?;

        let name = "twitter";
        let value = "loaf0bred";

        suite
            .account
            .add_text_record(token_id, TextRecord::new(name, value))?;

        // query text record to see if verified is not set
        let res = suite.account.nft_info(token_id)?;
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        suite
            .account
            .verify_text_record(token_id, name, true)
            .unwrap_err();

        suite
            .account
            .call_as(&verifier)
            .verify_text_record(token_id, name, true)?;

        // query text record to see if verified is set
        let res = suite.account.nft_info(token_id)?;

        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, Some(true));

        Ok(())
    }
    #[test]
    fn test_verify_false() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        mock.wait_seconds(1)?;

        let admin_user = mock.sender.clone();
        let verifier = mock.addr_make("verifier");
        let token_id = "bobo";

        suite.mint_and_list(mock, token_id, &admin_user)?;

        let name = "twitter";
        let value = "loaf0bred";

        suite
            .account
            .add_text_record(token_id, TextRecord::new(name, value))?;

        suite
            .account
            .call_as(&verifier)
            .verify_text_record(token_id, name, false)?;

        // query text record to see if verified is not set
        let res = suite.account.nft_info(token_id)?;

        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, Some(false));

        Ok(())
    }
    #[test]
    fn test_verified_text_record() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        mock.wait_seconds(1)?;

        let admin_user = mock.sender.clone();
        let token_id = "bobo";

        suite.mint_and_list(mock, token_id, &admin_user)?;

        let name = "twitter";
        let value = "loaf0bred";

        suite
            .account
            .add_text_record(token_id, TextRecord::new(name, value))?;

        // query text record to see if verified is not set
        let res = suite.account.nft_info(token_id)?;
        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        // attempt update text record w verified value
        suite.account.update_text_record(
            token_id,
            TextRecord {
                name: token_id.into(),
                value: "some new value".to_string(),
                verified: Some(true),
            },
        )?;

        // query text record to see if verified is set
        let res = suite.account.nft_info(token_id)?;

        assert_eq!(res.extension.records[0].name, name.to_string());
        assert_eq!(res.extension.records[0].verified, None);

        // query image nft
        assert_eq!(suite.account.image_nft(token_id)?, None);
        Ok(())
    }
    #[test]
    fn test_transfer_nft() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.sender.clone();
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), token_id, &admin_user)?;

        suite
            .account
            .transfer_nft(mock.addr_make("new-addr"), token_id)?;

        Ok(())
    }
    #[test]
    fn test_send_nft() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.sender.clone();
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), token_id, &admin_user)?;

        suite
            .account
            .send_nft(mock.addr_make("new-addr"), to_json_binary("ini")?, token_id)?;
        Ok(())
    }
    #[test]
    fn test_transfer_nft_and_bid() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;
        let bidder1 = mock.addr_make("bidder1");
        let market = suite.market.address()?;

        let user1 = mock.addr_make("user1");
        let admin_user = mock.sender.clone();
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), token_id, &admin_user)?;

        suite.account.transfer_nft(user1.clone(), token_id)?;

        suite.bid_w_funds(mock.clone(), &token_id, bidder1.clone(), BID_AMOUNT * 3)?;

        // user2 must approve the marketplace to transfer their name
        suite
            .account
            .call_as(&user1)
            .approve(market, token_id, None)?;
        // accept bid
        suite
            .market
            .call_as(&user1)
            .accept_bid(bidder1, token_id.into())?;

        Ok(())
    }
    #[test]
    fn test_transfer_nft_with_reverse_map() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let user = mock.addr_make("user");
        let user2 = mock.addr_make("user2");
        let admin_user = mock.sender.clone();
        let token_id = "bobo";

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), token_id, &user)?;

        suite
            .account
            .call_as(&user)
            .associate_address(token_id, Some(user.to_string()))?;

        assert_eq!(suite.account.account(user.clone())?, token_id);

        suite
            .account
            .call_as(&user)
            .transfer_nft(user2.clone(), token_id)?;

        suite.account.account(user).unwrap_err();
        suite.account.account(user2).unwrap_err();

        Ok(())
    }
    // #[test]
    // fn test_burn_nft() -> anyhow::Result<()> {
    //     Ok(())
    // }
    // #[test]
    // fn test_burn_with_existing_bids() -> anyhow::Result<()> {
    //     Ok(())
    // }
    // #[test]
    // fn test_burn_nft_with_reverse_map() -> anyhow::Result<()> {
    //     Ok(())
    // }
    #[test]
    fn test_sudo_update() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let max_record_count = suite.account.params()?.max_record_count;

        // run sudo msg
        mock.app.borrow_mut().sudo(SudoMsg::Wasm(WasmSudo {
            contract_addr: suite.account.address()?,
            message: to_json_binary(&btsg_account::account::SudoMsg::UpdateParams {
                max_record_count: max_record_count + 1,
            })?,
        }))?;

        assert_eq!(
            suite.account.params()?.max_record_count,
            max_record_count + 1
        );

        Ok(())
    }
}
mod public_start_time {

    use super::*;

    #[test]
    fn test_mint_before_start() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.sender.clone();
        let token_id = "bobo";
        let user4 = mock.addr_make("user4");

        suite
            .mint_and_list(mock.clone(), token_id, &admin_user)
            .unwrap_err();
        suite
            .mint_and_list(mock.clone(), token_id, &user4)
            .unwrap_err();
        Ok(())
    }
    #[test]
    fn test_update_start_time() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let res = suite.minter.config()?;
        assert_eq!(
            res.public_mint_start_time,
            mock.block_info()?.time.plus_seconds(1)
        );

        suite.minter.update_config(Config {
            public_mint_start_time: mock.block_info()?.time.plus_seconds(2),
        })?;

        let res = suite.minter.config()?;
        assert_eq!(
            res.public_mint_start_time,
            mock.block_info()?.time.plus_seconds(2)
        );

        Ok(())
    }
}

mod associate_address {

    use btsg_account::account::{Bs721AccountsQueryMsgFns, ExecuteMsgFns, InstantiateMsg};

    use super::*;

    #[test]
    fn test_transfer_to_eoa() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.sender.clone();

        let cw721_id = suite.account.code_id()?;
        let token_id = "bobo";

        let nft_addr = mock
            .instantiate(
                cw721_id,
                &InstantiateMsg {
                    verifier: None,
                    marketplace: suite.market.address()?,
                    base_init_msg: bs721_base::InstantiateMsg {
                        name: "test2".into(),
                        symbol: "TEST2".into(),
                        uri: None,
                        minter: suite.minter.address()?.to_string(),
                    },
                },
                "test".into(),
                Some(&admin_user),
                &[],
            )?
            .instantiated_contract_address()?;

        mock.wait_seconds(1)?;
        // mint and transfer to collection
        suite.mint_and_list(mock.clone(), &token_id, &admin_user)?;
        suite.account.transfer_nft(nft_addr.clone(), token_id)?;
        assert_eq!(suite.account.owner_of(token_id, None)?.owner, nft_addr);

        Ok(())
    }
    #[test]
    fn test_associate_with_a_contract_with_no_admin() -> anyhow::Result<()> {
        // For the purposes of this test, a collection contract with no admin needs to be instantiated (contract_with_no_admin)
        // This contract needs to have a creator that is itself a contract and this creator contract should have an admin (USER).
        // The admin (USER) of the creator contract will mint a name and associate the name with the collection contract that doesn't have an admin successfully.
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.sender.clone();

        let cw721_id = suite.account.code_id()?;

        let token_id = "bobo";
        // Instantiating the creator contract with an admin (USER)
        let creator_addr = mock
            .instantiate(
                cw721_id,
                &InstantiateMsg {
                    verifier: None,
                    marketplace: suite.market.address()?,
                    base_init_msg: bs721_base::InstantiateMsg {
                        name: "test2".into(),
                        symbol: "TEST2".into(),
                        uri: None,
                        minter: suite.minter.address()?.to_string(),
                    },
                },
                "test".into(),
                Some(&admin_user),
                &[],
            )?
            .instantiated_contract_address()?;

        // The creator contract instantiates the collection contract with no admin
        let collection_with_no_admin_addr = mock
            .call_as(&creator_addr)
            .instantiate(
                cw721_id,
                &InstantiateMsg {
                    verifier: None,
                    marketplace: suite.market.address()?,
                    base_init_msg: bs721_base::InstantiateMsg {
                        name: "test2".into(),
                        symbol: "TEST2".into(),
                        uri: None,
                        minter: suite.minter.address()?.to_string(),
                    },
                },
                "test".into(),
                None,
                &[],
            )?
            .instantiated_contract_address()?;

        mock.wait_seconds(1)?;
        // USER4 mints a name
        suite.mint_and_list(mock.clone(), &token_id, &admin_user)?;

        // USER4 tries to associate the name with the collection contract that doesn't have an admin
        suite
            .account
            .call_as(&admin_user)
            .associate_address(token_id, Some(collection_with_no_admin_addr.to_string()))?;

        mock.wait_seconds(1)?;
        Ok(())
    }
    #[test]
    fn test_associate_with_a_contract_with_no_admin_fail() -> anyhow::Result<()> {
        // For the purposes of this test, a collection contract with no admin needs to be instantiated (contract_with_no_admin)
        // This contract needs to have a creator that is itself a contract and this creator contract should have an admin (USER).
        // An address other than the admin (USER) of the creator contract will mint a name, try to associate the name with the collection contract that doesn't have an admin and fail.
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.addr_make("admin-user");
        let user4 = mock.addr_make("user4");

        let cw721_id = suite.account.code_id()?;

        let token_id = "bobo";
        // Instantiating the creator contract with an admin (USER)
        let creator_addr = mock
            .instantiate(
                cw721_id,
                &InstantiateMsg {
                    verifier: None,
                    marketplace: suite.market.address()?,
                    base_init_msg: bs721_base::InstantiateMsg {
                        name: "test2".into(),
                        symbol: "TEST2".into(),
                        uri: None,
                        minter: suite.minter.address()?.to_string(),
                    },
                },
                "test".into(),
                Some(&admin_user),
                &[],
            )?
            .instantiated_contract_address()?;

        // The creator contract instantiates the collection contract with no admin
        let collection_with_no_admin_addr = mock
            .call_as(&creator_addr)
            .instantiate(
                cw721_id,
                &InstantiateMsg {
                    verifier: None,
                    marketplace: suite.market.address()?,
                    base_init_msg: bs721_base::InstantiateMsg {
                        name: "test2".into(),
                        symbol: "TEST2".into(),
                        uri: None,
                        minter: suite.minter.address()?.to_string(),
                    },
                },
                "test".into(),
                None,
                &[],
            )?
            .instantiated_contract_address()?;

        mock.wait_seconds(1)?;
        // USER4 mints a name
        suite.mint_and_list(mock.clone(), &token_id, &user4)?;

        // USER4 tries to associate the name with the collection contract that doesn't have an admin
        let err = suite
            .account
            .call_as(&user4)
            .associate_address(token_id, Some(collection_with_no_admin_addr.to_string()))
            .unwrap_err();

        assert_eq!(
            err.root().to_string(),
            bs721_account::ContractError::UnauthorizedCreatorOrAdmin {}.to_string()
        );
        Ok(())
    }
    #[test]
    fn test_associate_with_a_contract_with_an_admin_fail() -> anyhow::Result<()> {
        let mock = MockBech32::new("bitsong");
        let mut suite = BtsgAccountSuite::new(mock.clone());
        suite.default_setup(mock.clone(), None, Some(mock.sender.clone()))?;

        let admin_user = mock.addr_make("admin-user");
        let user4 = mock.addr_make("user4");

        let cw721_id = suite.account.code_id()?;

        let token_id = "bobo";
        // Instantiating the creator contract with an admin (USER)
        let contract = mock
            .instantiate(
                cw721_id,
                &InstantiateMsg {
                    verifier: None,
                    marketplace: suite.market.address()?,
                    base_init_msg: bs721_base::InstantiateMsg {
                        name: "test2".into(),
                        symbol: "TEST2".into(),
                        uri: None,
                        minter: suite.minter.address()?.to_string(),
                    },
                },
                "test".into(),
                Some(&admin_user),
                &[],
            )?
            .instantiated_contract_address()?;

        mock.wait_seconds(1)?;
        suite.mint_and_list(mock.clone(), &token_id, &user4)?;

        let err = suite
            .account
            .call_as(&user4)
            .associate_address(token_id, Some(contract.to_string()))
            .unwrap_err();

        assert_eq!(
            err.root().to_string(),
            bs721_account::ContractError::UnauthorizedCreatorOrAdmin {}.to_string()
        );
        Ok(())
    }
}
