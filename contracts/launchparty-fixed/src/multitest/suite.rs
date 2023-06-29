use bs721_royalties::msg::ContributorMsg;
use cosmwasm_std::{Addr, Coin, Empty, Timestamp};
use cw_multi_test::{App, Contract, ContractWrapper, Executor, AppResponse};
use derivative::Derivative;
use anyhow::{anyhow, Result as AnyResult};

use bs721_base::msg::{
    ExecuteMsg as Bs721BaseExecuteMsg, InstantiateMsg as Bs721BaseInstantiateMsg,
};
use bs721_royalties::msg::{
    ExecuteMsg as Bs721RoyaltiesExecuteMsg, InstantiateMsg as Bs721RoyaltiesInstantiateMsg,
};

use crate::msg::{ConfigResponse, InstantiateMsg, PartyType, QueryMsg, ExecuteMsg};

pub const CREATOR: &str = "creator";
pub const SENDER: &str = "sender";

/// Helper function to create a wrapper around the bs721 base contract
pub fn contract_bs721_base() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new_with_empty(
        bs721_base::entry::execute,
        bs721_base::entry::instantiate,
        bs721_base::entry::query,
    ))
}

/// Helper function to create a wrapper around the bs721 royalties contract
pub fn contract_bs721_royalties() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new_with_empty(
        bs721_royalties::contract::execute,
        bs721_royalties::contract::instantiate,
        bs721_royalties::contract::query,
    ))
}

/// Helper function to create a wrapper around the launchparty contract
pub fn contract_launchparty() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply_empty(crate::contract::reply),
    )
}

// -------------------------------------------------------------------------------------------------
// TestSuiteBuilder
// -------------------------------------------------------------------------------------------------

/// Stores genesis configuration for tests. This strcture is used initialize a TestSuite.
#[derive(Derivative, Debug)]
#[derivative(Default(new = "true"))]
pub struct TestSuiteBuilder {
    /// Creator of the collection. If not provided it will be the sender.
    #[derivative(Default(value = "String::from(CREATOR)"))]
    pub creator: String,
    /// BS721 token name.
    #[derivative(Default(value = "String::from(\"album\")"))]
    pub name: String,
    /// BS721 token symbol.
    #[derivative(Default(value = "String::from(\"album\")"))]
    pub symbol: String,
    /// Price of single nft minting.
    pub price: Coin,
    /// BS721 token uri.
    pub base_token_uri: String,
    /// BS721 collection uri.
    pub collection_uri: String,
    pub seller_fee_bps: u16,
    pub referral_fee_bps: u16,
    /// Contributors to the collection.
    pub contributors: Vec<ContributorMsg>,
    /// Start time of the launchparty.
    pub start_time: Timestamp,
    /// End condition of the collection launchparty.
    #[derivative(Default(value = "PartyType::MaxEdition(1)"))]
    pub party_type: PartyType,
}

impl TestSuiteBuilder {
    /// Helper function to define the price of the bs721 collection.
    pub fn with_price(mut self, price: Coin) -> Self {
        self.price = price;
        self
    }

    /// Helper function to define the price of the bs721 collection.
    pub fn with_starttime(mut self, start_time: Timestamp) -> Self {
        self.start_time = start_time;
        self
    }

    /// Helper function to define the end condition of the launchparty.
    pub fn with_party_type(mut self, party_type: PartyType) -> Self {
        self.party_type = party_type;
        self
    }

    /// Helper function to add contributors to the contract.
    pub fn with_contributors(
        mut self,
        contributors: impl IntoIterator<Item = ContributorMsg>,
    ) -> Self {
        self.contributors.extend(contributors.into_iter());
        self
    }

    /// Helper function to instantiate the launchparty contract with parameters defined by the TestSuiteBuilder
    pub fn instantiate_launchparty(
        self,
        app: &mut App,
        code_id: u64,
        bs721_token_code_id: u64,
        bs721_royalties_code_id: u64,
    ) -> Addr {
        // could we also use mem to optimize code and avoid clone
        let init_msg = InstantiateMsg {
            creator: Some(self.creator.clone()),
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            price: self.price.clone(),
            base_token_uri: self.base_token_uri.clone(),
            collection_uri: self.collection_uri.clone(),
            seller_fee_bps: self.seller_fee_bps.clone(),
            referral_fee_bps: self.referral_fee_bps.clone(),
            contributors: self.contributors.clone(),
            start_time: self.start_time,
            party_type: self.party_type.clone(),
            bs721_token_code_id,
            bs721_royalties_code_id,
        };

        app.instantiate_contract(
            code_id,
            Addr::unchecked(self.creator.clone()),
            &init_msg,
            &[],
            "Launchparty",
            None,
        )
        .unwrap()
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app: App = App::default();

        let bs721_base_code_id = app.store_code(contract_bs721_base());
        let bs721_royalties_code_id = app.store_code(contract_bs721_royalties());
        let launchparty_code_id = app.store_code(contract_launchparty());

        let contract_address = self.instantiate_launchparty(
            &mut app,
            launchparty_code_id,
            bs721_base_code_id,
            bs721_royalties_code_id,
        );

        Suite {
            app,
            contract_address,
            bs721_base_code_id,
            bs721_royalties_code_id,
        }
    }
}

/// Test suite
pub struct Suite {
    /// The multitest app
    app: App,
    /// Address of the launchparty contract.
    contract_address: Addr,
    /// Code id used to instantiate a bs721 token contract.
    bs721_base_code_id: u64,
    /// Code id used to instantiate bs721 royalties contract.
    bs721_royalties_code_id: u64,
}

impl Suite {
    pub fn app(&mut self) -> &mut App {
        &mut self.app
    }

    fn contract_address(&self) -> Addr {
        self.contract_address.clone()
    }

    /// Helper function to mint a bs721 token. The sender is defined as a const.
    pub fn mint(&mut self, referral: Option<String>) -> AnyResult<AppResponse> {

        let msg = ExecuteMsg::Mint { referral };

        self.app.execute_contract(Addr::unchecked(SENDER), self.contract_address(), &msg, &[])
    } 

    /// Helper function to query launchparty contract configuration.
    pub fn query_config(&self) -> ConfigResponse {
        self.app
            .wrap()
            .query_wasm_smart(self.contract_address(), &QueryMsg::GetConfig {})
            .unwrap()
    }
}
