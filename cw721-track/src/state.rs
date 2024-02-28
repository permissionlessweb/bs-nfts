use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::Item;

use crate::Extension;

#[cw_serde]
pub struct Config {
    pub next_token_id: u32,
    pub royalties_address: Option<Addr>,
    pub seller_fee_bps: Option<u16>,
}

pub struct Cw721TrackContract<'a> {
    pub config: Item<'a, Config>,
    pub cw721_contract: cw721_base::Cw721Contract<'a, Extension, Empty, Empty, Empty>,
}

impl Default for Cw721TrackContract<'static> {
    fn default() -> Self {
        Self {
            config: Item::new("config"),
            cw721_contract: cw721_base::Cw721Contract::default(),
        }
    }
}
