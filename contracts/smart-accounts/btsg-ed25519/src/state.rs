use cw_storage_plus::Item;
use saa::types::PasskeyPayload;

/// Stores the contract's configuration
pub const PAYLOAD: Item<PasskeyPayload> = Item::new("pkp");
