use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp, Uint128, StdResult, Coin};

use crate::ContractError;

/// Contributor info.
#[cw_serde]
pub struct Contributor {
    /// Address of the contributor.
    pub addr: String,
    pub weight: u64,
}

/// Possible launchparty type. Each type defines how the party end.
#[cw_serde]
pub enum PartyType {
    /// Maximum number of mintable tokens.
    MaxEdition(u32),
    /// Number of blocks for which tokens are mintable.
    EndTime(u32),
}

#[cw_serde]
pub struct InstantiateMsg {
    /// Creator of the collection. If not provided it will be the sender.
    pub creator: Option<String>,
    /// Price of single nft minting
    pub price: Coin,
    /// BS721 token name
    pub name: String,
    /// BS721 token symbol
    pub symbol: String,
    /// BS721 token uri
    pub base_token_uri: String,
    pub collection_uri: String,
    pub seller_fee_bps: u16,
    /// Contributors to the collection.
    pub contributors: Vec<Contributor>,
    /// Start time of the launchparty
    pub start_time: Timestamp,
    /// End condition of the collection launchparty.
    pub party_type: PartyType,
    /// Code id to be usede to instantiate a bs721 token contract.
    pub bs721_token_code_id: u64,
    /// Code id to be used to instantiate bs721 royalty contract
    pub bs721_royalty_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Retrieves contract's configuration
    #[returns(ConfigResponse)]
    GetConfig {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub creator: Addr,
    pub bs721_address: Option<Addr>,
    pub price: Uint128,
    pub name: String,
    pub symbol: String,
    pub base_token_uri: String,
    pub next_token_id: u32,
    pub seller_fee_bps: u16,
    pub royalty_address: Option<Addr>,
    pub start_time: Timestamp,
    pub party_type: PartyType,
}

impl PartyType {
    /// Performs basic validation checks on an istance of this type.
    pub fn validate(&self) -> Result<(), ContractError>{
        match self {
            PartyType::MaxEdition(number) => {
                if number == &0u32 {
                    return Err(ContractError::ZeroEditions {})
                }
            }
            PartyType::EndTime(duration) => {
                if duration == &0u32 {
                    return Err(ContractError::ZeroDuration {})
                }
            }
        }
        Ok(())
    }
}