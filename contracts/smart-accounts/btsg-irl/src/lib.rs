pub mod contract;
mod error;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    symbol: String,
    name: String,
    max_supply: Uint128,
    uri: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Mint fantokens to recipients
    MintFantokens { data: Vec<MintTicketObject> },
    /// Update the URI for the fantoken
    SetUri { uri: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get fantoken information
    #[returns(Option<FantokenInfo>)]
    GetFantokenInfo {},
    /// Get total minted amount
    #[returns(Uint128)]
    GetMintedAmount {},
}

#[cw_serde]
pub struct MintTicketObject {
    pub ticket: String, // recipient address
    pub amount: u64,    // amount to mint
}

#[cw_serde]
pub struct FantokenInfo {
    pub symbol: String,
    pub name: String,
    pub max_supply: Uint128,
    pub authority: String,
    pub uri: String,
    pub minter: String,
    pub denom: String,
}
