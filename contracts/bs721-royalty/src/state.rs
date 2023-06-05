use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Contributor {
    pub role: String,
    pub share: u32,
}

pub const CONTRIBUTORS: Map<&Addr, Contributor> = Map::new("contributors");
pub const TOTAL_SHARES: Item<u64> = Item::new("total_shares");
pub const DENOM: Item<String> = Item::new("denom");
