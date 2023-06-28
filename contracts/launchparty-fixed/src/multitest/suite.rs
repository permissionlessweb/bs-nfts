use bs721_royalties::msg::ContributorMsg;
use cosmwasm_std::{Empty, Coin, Timestamp};
use cw_multi_test::{ContractWrapper, Contract};
use derivative::Derivative;

use crate::msg::PartyType;


/// Helper function to create a wrapper around the bs721 base contract
pub fn contract_bs721_base() ->  Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new_with_empty(
            bs721_base::entry::execute,
            bs721_base::entry::instantiate,
            bs721_base::entry::query,
        )
    )
}

pub fn contract_bs721_royalties() -> Box<dyn Contract<Empty>> {
    Box::new(
        ContractWrapper::new_with_empty(
            crate::contract::execute, 
            crate::contract::instantiate, 
            crate::contract::query,
    ).with_reply_empty(crate::contract::reply)
    )
}

pub const CREATOR: &str = "creator";

// -------------------------------------------------------------------------------------------------
// TestSuiteBuilder
// -------------------------------------------------------------------------------------------------

/// Stores genesis configuration for tests. This strcture is used initialize a TestSuite.
#[derive(Derivative, Debug)]
#[derivative(Default(new="true"))]
pub struct TestSuiteBuilder {
    /// Creator of the collection. If not provided it will be the sender.
    #[derivative(Default(value = "Some(String::from(CREATOR))"))]
    pub creator: Option<String>,
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
    /// Code id used to instantiate a bs721 token contract.
    pub bs721_token_code_id: u64,
    /// Code id used to instantiate bs721 royalties contract.
    pub bs721_royalties_code_id: u64,
}

impl TestSuiteBuilder {
    
}