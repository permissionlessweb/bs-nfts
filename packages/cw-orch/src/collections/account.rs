use cw_orch::{interface, prelude::*};

use bs721_account::msg::{Bs721AccountsQueryMsg as QueryMsg, ExecuteMsg, InstantiateMsg};
use bs721_account::{execute, instantiate, query, sudo};
use btsg_account::Metadata;

#[interface(InstantiateMsg, ExecuteMsg::<Metadata>, QueryMsg, Empty)]
pub struct BitsongAccountCollection;

impl<Chain> Uploadable for BitsongAccountCollection<Chain, Metadata> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("bs721_account")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_sudo(sudo))
    }
}
