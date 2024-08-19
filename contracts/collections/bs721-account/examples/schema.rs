use btsg_account::Metadata;
use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;

use bs721_account::{
    msg::{ExecuteMsg, InstantiateMsg},
    QueryMsg,
};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg<Metadata>,
        query: QueryMsg,
    }
}
