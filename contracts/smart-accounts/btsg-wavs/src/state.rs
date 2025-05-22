use cosmwasm_std::Binary;
use cw_storage_plus::Item;

/// Stores the contract's configuration
pub const WAVS_PUBKEY: Item<Vec<Binary>> = Item::new("wavs");

#[cosmwasm_schema::cw_serde]
pub struct BlsMetadata {
    pub wavs_operator_avs_keys: Vec<Binary>,
}
