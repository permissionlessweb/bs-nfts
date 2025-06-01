use cw_orch::{interface, prelude::*};

use btsg_wavs::contract::{execute, instantiate, query, sudo};
use btsg_wavs::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

/// Uploadable trait for bs721_account_minter & use with cw-orchestrator library
#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct BtsgWavsAuthenticator;

impl<Chain> Uploadable for BtsgWavsAuthenticator<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("btsg_wavs")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_sudo(sudo))
    }
}
