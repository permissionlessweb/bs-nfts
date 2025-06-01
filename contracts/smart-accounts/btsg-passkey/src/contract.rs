use btsg_auth::{
    AuthenticationRequest, ConfirmExecutionRequest, OnAuthenticatorAddedRequest,
    OnAuthenticatorRemovedRequest, TrackRequest,
};
use cosmwasm_std::{from_json, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use saa_common::Verifiable;

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg},
    state::PAYLOAD,
    ContractError,
};

use cosmwasm_std::entry_point;

use saa::PasskeyCredential;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:btsg-wavs";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Can only be called by governance
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    PAYLOAD.save(deps.storage, &msg.payload)?;
    cw_ownable::initialize_owner(
        deps.storage,
        deps.api,
        Some(msg.owner.unwrap_or(info.sender).as_str()),
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        btsg_auth::AuthenticatorSudoMsg::OnAuthenticatorAdded(auth_add) => {
            sudo_on_authenticator_added_request(deps, auth_add)
        }
        btsg_auth::AuthenticatorSudoMsg::OnAuthenticatorRemoved(auth_remove) => {
            sudo_on_authenticator_removed_request(deps, auth_remove)
        }
        btsg_auth::AuthenticatorSudoMsg::Authenticate(auth_req) => {
            sudo_authentication_request(deps, auth_req)
        }
        btsg_auth::AuthenticatorSudoMsg::Track(track_req) => sudo_track_request(deps, track_req),
        btsg_auth::AuthenticatorSudoMsg::ConfirmExecution(conf_exec_req) => {
            sudo_confirm_execution_request(deps, conf_exec_req)
        }
    }
}

fn sudo_on_authenticator_added_request(
    _deps: DepsMut,
    auth_added: OnAuthenticatorAddedRequest,
) -> Result<Response, ContractError> {
    // small storage writes, for example global contract entropy or count of registered accounts
    match auth_added.authenticator_params {
        Some(_) => Ok(Response::new().add_attribute("action", "auth_added_req")),
        None => Err(ContractError::Unauthorized {}),
    }
}

fn sudo_on_authenticator_removed_request(
    _deps: DepsMut,
    _auth_removed: OnAuthenticatorRemovedRequest,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "auth_removed_req"))
}

fn sudo_authentication_request(
    deps: DepsMut,
    auth_req: Box<AuthenticationRequest>,
) -> Result<Response, ContractError> {
    let cred: PasskeyCredential = from_json(&auth_req.signature)?;

    // assert client origin is same as one registered
    // if let Some(origin) = PAYLOAD.load(deps.storage)?.origin {
    //     if cred.client_data.origin != origin {
    //         return Err(ContractError::Unauthorized {});
    //     }
    // }

    // verify passkey request
    cred.verify_cosmwasm(deps.api)?;

    Ok(Response::new().add_attribute("action", "auth_req"))
}

fn sudo_track_request(
    _deps: DepsMut,
    TrackRequest { .. }: TrackRequest,
) -> Result<Response, ContractError> {
    // this is where we handle any processes after authentication, regarding message contents, prep to track balances prior to msg execution, etc..
    Ok(Response::new().add_attribute("action", "track_req"))
}

fn sudo_confirm_execution_request(
    _deps: DepsMut,
    _confirm_execution_req: ConfirmExecutionRequest,
) -> Result<Response, ContractError> {
    // here is were we compare balances post event execution, based on data saved from sudo_track_request,etc..
    Ok(Response::new().add_attribute("action", "conf_exec_req"))
}

pub fn execute_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    action: cw_ownable::Action,
) -> Result<Response, ContractError> {
    let ownership = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
    Ok(Response::default().add_attributes(ownership.into_attributes()))
}
