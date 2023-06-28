use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::Item;

use crate::msg::PartyType;

/// Smart contract configuration structure.
#[cw_serde]
pub struct Config {
    pub creator: Addr,
    pub name: String,
    pub symbol: String,
    pub price: Coin,
    pub base_token_uri: String,
    pub next_token_id: u32,
    pub seller_fee_bps: u16,
    pub referral_fee_bps: u16,
    pub start_time: Timestamp,
    pub party_type: PartyType,
    pub bs721_address: Option<Addr>,
    pub royalties_address: Option<Addr>,
}

/// Stores the contract's configuration
pub const CONFIG: Item<Config> = Item::new("config");
