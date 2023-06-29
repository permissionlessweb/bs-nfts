use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::Item;
use cw_utils::Duration;

#[cw_serde]
pub struct Config {
    pub creator: Addr,                 //
    pub bs721_address: Option<Addr>,   //
    pub max_editions: u32,             //
    pub price: Coin,                   //
    pub name: String,                  //
    pub symbol: String,                //
    pub base_token_uri: String,        //
    pub next_token_id: u32,            //
    pub seller_fee_bps: u16,           //
    pub royalty_address: Option<Addr>, //
    pub referral_fee_bps: u16,         //
    pub start_time: Timestamp,         //
    pub duration: Duration,            //
}

pub const CONFIG: Item<Config> = Item::new("config");
