use btsg_account::account::SudoParams;
use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};

pub type TokenUri = Addr;
pub type TokenId = String;

/// Address of the text record verification oracle
pub const REVERSE_MAP: Map<&TokenUri, TokenId> = Map::new("map");
pub const VERIFIER: Admin = Admin::new("verifier");
pub const SUDO_PARAMS: Item<SudoParams> = Item::new("params");
pub const ACCOUNT_MARKETPLACE: Item<Addr> = Item::new("account-marketplace");
