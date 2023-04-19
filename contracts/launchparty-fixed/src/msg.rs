use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp, Uint128};

#[cw_serde]
pub struct Contributor {
    pub addr: String,
    pub weight: u64,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub creator: Addr,
    pub max_editions: u32,
    pub price: Uint128,
    pub name: String,
    pub symbol: String,
    pub base_token_uri: String,
    pub collection_uri: String,
    pub seller_fee_bps: u16,
    pub contributors: Vec<Contributor>,
    pub start_time: Timestamp,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    GetConfig {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub creator: Addr,
    pub bs721_address: Option<Addr>,
    pub max_editions: u32,
    pub price: Uint128,
    pub name: String,
    pub symbol: String,
    pub base_token_uri: String,
    pub next_token_id: u32,
    pub seller_fee_bps: u16,
    pub royalty_address: Option<Addr>,
    pub start_time: Timestamp,
}
