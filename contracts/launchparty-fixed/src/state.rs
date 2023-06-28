use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128, Coin};
use cw_storage_plus::Item;

use crate::msg::PartyType;

/// Smart contract configuration structure.
#[cw_serde]
pub struct Config {
    pub creator: Addr,
    pub bs721_address: Option<Addr>,
    pub price: Coin,
    pub name: String,
    pub symbol: String,
    pub base_token_uri: String,
    pub next_token_id: u32,
    pub seller_fee_bps: u16,
    pub royalty_address: Option<Addr>,
    pub start_time: Timestamp,
    pub party_type: PartyType,
}

/// Stores the contract's configuration
pub const CONFIG: Item<Config> = Item::new("config");
