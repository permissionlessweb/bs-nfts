use bs721_base::InstantiateMsg;
use bs721_launchparty::msg::{
    ExecuteMsg as LaunchpartyExecuteMsg, ExecuteMsgFns as _, InstantiateMsg as Bs721LaunchInitMsg,
    PartyType, QueryMsgFns as _,
};
use btsg_nft_scripts::BtsgNftSuite;
use cosmwasm_std::{coin, Addr, Coin, Timestamp, Uint128};
use cw_orch::{
    mock::{cw_multi_test::AppResponse, MockBech32},
    prelude::*,
};

pub struct TestLaunchpartySuite<MockBech32>
where
    MockBech32: cw_orch::prelude::CwEnv,
{
    pub chain: MockBech32,
    pub btsg: BtsgNftSuite<MockBech32>,
    /// Creator of the collection. If not provided it will be the sender.
    pub creator: Addr,
    /// BS721 token symbol.
    pub symbol: String,
    pub name: String,
    pub uri: String,
    /// Price of single nft minting.
    pub price: Coin,
    /// Maximum numer of tokens an address can mint
    pub max_per_address: Option<u32>,
    /// BS721 token uri.
    pub seller_fee_bps: u16,
    pub referral_fee_bps: u16,
    pub protocol_fee_bps: u16,
    /// Start time of the launchparty.
    pub start_time: Timestamp,
    /// End condition of the collection launchparty.
    pub party_type: PartyType,
    pub init_funds: Vec<(Addr, Vec<Coin>)>,
}

impl TestLaunchpartySuite<MockBech32> {
    pub fn new() -> anyhow::Result<Self> {
        let chain = MockBech32::new("bitsong");
        let suite = BtsgNftSuite::new(chain.clone());
        let creator = chain.addr_make("creator");
        Ok(TestLaunchpartySuite {
            chain: chain,
            btsg: suite,
            creator: creator.clone(),
            symbol: "album".into(),
            name: "name".into(),
            uri: "uri".into(),
            price: coin(1_000, "ubtsg"),
            max_per_address: None,
            seller_fee_bps: 0,
            referral_fee_bps: 0,
            protocol_fee_bps: 0,
            start_time: Timestamp::from_seconds(1571797419),
            party_type: PartyType::MaxEdition(1),
            init_funds: vec![],
        })
    }

    /// Helper function to initialize the bank module with funds associated to particular addresses.
    pub fn with_funds(mut self, addr: &str, funds: &[Coin]) -> Self {
        self.init_funds.push((Addr::unchecked(addr), funds.into()));
        self
    }

    /// Helper function to define the price of the bs721 collection.
    pub fn with_price(mut self, price: Coin) -> Self {
        self.price = price;
        self
    }
    /// Helper function to mint a bs721 token. The sender is defined as a const.
    pub fn mint(
        &mut self,
        sender: impl ToString,
        referral: Option<String>,
        amount: u32,
        funds: Option<Coin>,
    ) -> Result<AppResponse, CwOrchError> {
        let msg = LaunchpartyExecuteMsg::Mint { referral, amount };

        let send_funds: Vec<Coin> = funds.map_or_else(Vec::new, |sent_coin| vec![sent_coin]);

        self.btsg
            .launchparty
            .call_as(&Addr::unchecked(sender.to_string()))
            .execute(&msg, &send_funds)
    }
    pub fn build(self) -> anyhow::Result<Self> {
        self.btsg.upload()?;
        let res = self.btsg.launchparty.call_as(&self.creator).instantiate(
            &Bs721LaunchInitMsg {
                symbol: self.symbol.clone(),
                name: self.name.clone(),
                uri: self.uri.clone(),
                price: self.price.clone(),
                max_per_address: self.max_per_address.clone(),
                payment_address: self.creator.to_string(),
                seller_fee_bps: self.seller_fee_bps,
                referral_fee_bps: self.referral_fee_bps,
                protocol_fee_bps: self.protocol_fee_bps,
                start_time: self.start_time,
                party_type: self.party_type.clone(),
                bs721_code_id: self.btsg.base.code_id()?,
                bs721_admin: self.chain.sender_addr().to_string(),
            },
            None,
            &[],
        )?;
        // grab bs721 addr from init attributes
        self.btsg.base.set_address(&Addr::unchecked(
            res.event_attr_value("wasm", "bs721_addr")?,
        ));

        // setup default balances
        for (addr, coin) in self.init_funds.clone() {
            println!("addr: {}", addr);
            self.chain.add_balance(&addr, coin)?;
        }

        // workaround for cosmwasm-std test state (contract IS instantiated by launchparty contract, but cannot be found in test state [persists in both cw-orch & unit tests])
        let nft_addr = self
            .btsg
            .base
            .instantiate(
                &InstantiateMsg {
                    name: self.name.clone(),
                    symbol: self.symbol.clone(),
                    uri: Some(self.uri.clone()),
                    minter: self.btsg.launchparty.addr_str()?,
                },
                None,
                &[],
            )?
            .instantiated_contract_address()?;
        self.btsg
            .launchparty
            .call_as(&self.creator)
            .set_nft_address(nft_addr)?;

        println!("launchpad contract: {}", self.btsg.launchparty.addr_str()?);
        println!("bs721-base contract: {}", self.btsg.base.addr_str()?);
        let binding = self.chain.clone().state().get_all_addresses()?;
        let res: Vec<(&String, &Addr)> = binding.iter().map(|a| a).collect();

        println!("res {:#?}!", res);
        Ok(self)
    }
}

#[test]
fn instantiate() -> anyhow::Result<()> {
    let suite = TestLaunchpartySuite::new()?.build()?;
    let resp = suite.btsg.launchparty.get_config()?;
    assert_eq!(resp.bs721_address, suite.btsg.base.address()?);
    assert_eq!(resp.payment_address, suite.creator);

    Ok(())
}

#[test]
fn mint_single_no_referral() -> anyhow::Result<()> {
    let minter = "bitsong1h6t805h2vjfzpa3m9n8kyadyng9xf604nhvev8tf5qdg65jh3ruq4z9ty9";
    let mut suite = TestLaunchpartySuite::new()?
        .with_funds(&minter.to_string(), &[coin(1_000, "ubtsg")])
        .with_price(coin(1, "ubtsg"))
        .build()?;

    suite.mint(minter, None, 1, Some(coin(1, "ubtsg")))?;

    let pay_recipient = suite.btsg.launchparty.get_config()?.payment_address;

    assert_eq!(
        suite.chain.query_balance(&pay_recipient, "ubtsg")?,
        Uint128::one()
    );

    Ok(())
}

// #[test]
// fn mint_multiple() -> anyhow::Result<()> {
//     let suite = BtsgNftTestSuite::new()?;
//     Ok(())
// }

// #[test]
// fn max_per_address() -> anyhow::Result<()> {
//     let suite = BtsgNftTestSuite::new()?;
//     Ok(())
// }

// #[test]
// fn query_max_per_address() -> anyhow::Result<()> {
//     let suite = BtsgNftTestSuite::new()?;
//     Ok(())
// }
