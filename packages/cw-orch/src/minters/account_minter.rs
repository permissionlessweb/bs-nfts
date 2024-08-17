use cw_orch::{interface, prelude::*};

use bs721_account_minter::contract::{execute, instantiate, query, reply, sudo};
use bs721_account_minter::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct BitsongAccountMinter;

impl<Chain> Uploadable for BitsongAccountMinter<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("bs721_account_marketplace")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(execute, instantiate, query)
                .with_reply(reply)
                .with_sudo(sudo),
        )
    }
}
