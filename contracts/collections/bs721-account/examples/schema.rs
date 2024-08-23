use btsg_account::{
    account::{ExecuteMsg, InstantiateMsg},
    Metadata,
};
use cosmwasm_schema::write_api;

use bs721_account::QueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg<Metadata>,
        query: QueryMsg,
    }
}
