use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, CustomQuery, Deps, StdError, StdResult, WasmMsg};
use cw_storage_plus::Item;
use thiserror::Error;

pub struct Admin(Item<Option<Addr>>);

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

impl Admin {
    /// Returns Ok(true) if this is an admin, Ok(false) if not and an Error if
    /// we hit an error with Api or Storage usage
    pub fn is_admin<Q: CustomQuery>(&self, deps: Deps<Q>, caller: &Addr) -> StdResult<bool> {
        match self.0.load(deps.storage)? {
            Some(owner) => Ok(caller == owner),
            None => Ok(false),
        }
    }

    /// Like is_admin but returns AdminError::NotAdmin if not admin.
    /// Helper for a nice one-line auth check.
    pub fn assert_admin<Q: CustomQuery>(
        &self,
        deps: Deps<Q>,
        caller: &Addr,
    ) -> Result<(), AdminError> {
        if !self.is_admin(deps, caller)? {
            Err(AdminError::NotAdmin {})
        } else {
            Ok(())
        }
    }
}

/// Errors returned from Admin
#[derive(Error, Debug, PartialEq)]
pub enum AdminError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Caller is not admin")]
    NotAdmin {},
}
