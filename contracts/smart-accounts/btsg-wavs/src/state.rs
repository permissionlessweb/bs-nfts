use cosmwasm_std::Binary;



#[cosmwasm_schema::cw_serde]
pub struct BlsMetadata {
    pub wavs_operator_avs_keys: Vec<Binary> 
}
