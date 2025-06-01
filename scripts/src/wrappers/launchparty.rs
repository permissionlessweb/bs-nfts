use cw_orch::{interface, prelude::*};

use bs721_launchparty::contract::{execute, instantiate, query, reply};
use bs721_launchparty::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

/// Uploadable trait for bs721_account_minter & use with cw-orchestrator library
#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct Btsg721Launchparty;

impl<Chain> Uploadable for Btsg721Launchparty<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("btsg721_launchparty")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(execute, instantiate, query).with_reply(reply), // .with_sudo(sudo),
        )
    }
}
