use bs721::InstantiateMsg;
use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;

use bs721_base::{ExecuteMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg<Empty, Empty>,
        query: QueryMsg<Empty>,
    }
}
