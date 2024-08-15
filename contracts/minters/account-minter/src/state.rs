use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Item;

use bs_account::minter::{Config, SudoParams};
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

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sp");

pub const ACCOUNT_COLLECTION: Item<Addr> = Item::new("ac");

pub const ACCOUNT_MARKETPLACE: Item<Addr> = Item::new("am");

pub const ADMIN: Admin = Admin::new("a");

/// Controls if minting is paused or not by admin
pub const PAUSED: Item<bool> = Item::new("paused");

pub const CONFIG: Item<Config> = Item::new("config");
