use cw_orch::{interface, prelude::*};

use bs721_account_marketplace::contract::{execute, instantiate, query, sudo};
use btsg_account::market::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct BitsongAccountMarketplace;

impl<Chain> Uploadable for BitsongAccountMarketplace<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("bs721_account_marketplace")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_sudo(sudo))
    }
}
