use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Item;

use bs_profile::minter::{Config, SudoParams};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WhitelistContract {
    pub contract_type: WhitelistContractType,
    pub addr: Addr,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum WhitelistContractType {
    UpdatableFlatrateDiscount,
    UpdatablePercentDiscount,
}

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("params");

pub const PROFILE_COLLECTION: Item<Addr> = Item::new("profile-collection");

pub const PROFILE_MARKETPLACE: Item<Addr> = Item::new("profile-market");

pub const ADMIN: Admin = Admin::new("admin");

/// Can only be updated by admin
pub const WHITELISTS: Item<Vec<WhitelistContract>> = Item::new("whitelists");

/// Controls if minting is paused or not by admin
pub const PAUSED: Item<bool> = Item::new("paused");

pub const CONFIG: Item<Config> = Item::new("config");
