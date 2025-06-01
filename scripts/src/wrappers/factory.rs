use cw_orch::{interface, prelude::*};

use bs721_factory::contract::{execute, instantiate, query};
use bs721_factory::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

/// Uploadable trait for bs721_account_minter & use with cw-orchestrator library
#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct Bs721Factory;

impl<Chain> Uploadable for Bs721Factory<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("bs721_factory")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query))
    }
}
