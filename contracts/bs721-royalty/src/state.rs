use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

/// Contributor's information
#[cw_serde]
pub struct Contributor {
    /// Contributor's role
    pub role: String,
    /// Contributor associated shares
    pub share: u32,
    /// Contributor withdrawable royalties
    pub withdrawable_amount: Uint128,
}

/// Stores contributors information with their address as keys.
pub const CONTRIBUTORS: Map<&Addr, Contributor> = Map::new("contributors");
/// Stores the total contributors shares value.
pub const TOTAL_SHARES: Item<u64> = Item::new("total_shares");
/// Stores the royalties token denom.
pub const DENOM: Item<String> = Item::new("denom");
/// Stores the total amount of tokens that can be withdrawn as royalties.
pub const WITHDRAWABLE_AMOUNT: Item<Uint128> = Item::new("withdrawable_amount");
