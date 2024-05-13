mod contract_tests;
mod error;
mod execute;
pub mod helpers;
pub mod msg;
mod query;
pub mod state;

pub use crate::error::ContractError;
pub use crate::msg::{ExecuteMsg, MintMsg, MinterResponse, QueryMsg};
pub use crate::state::Bs721Contract;
use cosmwasm_std::Empty;

// This is a simple type to let us handle empty extensions
pub type Extension = Option<Empty>;

// Version info for migration
pub const CONTRACT_NAME: &str = "crates.io:bs721-base";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod entry {
    use super::*;

    use bs721::InstantiateMsg;
    #[cfg(not(feature = "library"))]
    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

    // This makes a conscious choice on the various generics used by the contract
    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let res = Bs721Contract::<Extension, Empty, Empty, Empty>::default()
            .instantiate(deps, env, info, msg)?;

        Ok(res
            .add_attribute("contract_name", CONTRACT_NAME)
            .add_attribute("contract_version", CONTRACT_VERSION))
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<Extension, Empty>,
    ) -> Result<Response, ContractError> {
        Bs721Contract::<Extension, Empty, Empty, Empty>::default().execute(deps, env, info, msg)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg<Empty>) -> StdResult<Binary> {
        Bs721Contract::<Extension, Empty, Empty, Empty>::default().query(deps, env, msg)
    }
}
