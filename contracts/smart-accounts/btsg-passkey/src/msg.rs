use btsg_auth::AuthenticatorSudoMsg;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use saa::types::PasskeyPayload;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<Addr>,
    pub payload: PasskeyPayload,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {}

pub type SudoMsg = AuthenticatorSudoMsg;
