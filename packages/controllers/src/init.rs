use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, WasmMsg};

#[cw_serde]
pub struct ContractInstantiateMsg {
    pub code_id: u64,
    pub msg: Binary,
    pub admin: Option<String>,
    pub label: String,
}

impl ContractInstantiateMsg {
    pub fn into_wasm_msg(self, creator: Addr) -> WasmMsg {
        WasmMsg::Instantiate {
            admin: Some(self.admin.unwrap_or(creator.to_string())),
            code_id: self.code_id,
            msg: self.msg,
            label: self.label,
            funds: vec![],
        }
    }
}
