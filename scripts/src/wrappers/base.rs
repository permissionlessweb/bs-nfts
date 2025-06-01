use cw_orch::{interface, prelude::*};

use bs721::{Bs721ExecuteMsg, Bs721QueryMsg};
use bs721_base::{
    entry::{execute, instantiate, query},
    InstantiateMsg,
};

/// Uploadable trait for bs721_account_minter & use with cw-orchestrator library
#[interface(InstantiateMsg, Bs721ExecuteMsg<Empty,E>, Bs721QueryMsg, Empty)]
pub struct Bs721Base;

impl<Chain> Uploadable for Bs721Base<Chain, Empty, Empty> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("bs721_base")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query))
    }
}
