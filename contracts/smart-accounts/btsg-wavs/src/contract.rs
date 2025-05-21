use btsg_auth::{
    AuthenticationRequest, ConfirmExecutionRequest, OnAuthenticatorAddedRequest,
    OnAuthenticatorRemovedRequest, TrackRequest,
};
use cosmwasm_std::{from_json, DepsMut, Env, HashFunction, MessageInfo, Response};
use cw2::set_contract_version;

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, SudoMsg},
    state::BlsMetadata,
    ContractError,
};

use cosmwasm_std::entry_point;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:btsg-wavs";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Can only be called by governance
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new())
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
    deps: DepsMut,
    auth_added: OnAuthenticatorAddedRequest,
) -> Result<Response, ContractError> {
    // small storage writes, for example global contract entropy or count of registered accounts
    match auth_added.authenticator_params {
        Some(_) => Ok(Response::new().add_attribute("action", "auth_added_req")),
        None => Err(ContractError::MissingAuthenticatorMetadata {}),
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
    // EXAMPLE IMPLEMENTATION FOR BLS12_381 VERIFICATION

    // Aggregate public keys (G1 points)
    let raw_g1_points: Vec<_> = auth_req
        .signature_data
        .signers
        .iter()
        .map(|a| a.clone().as_str().as_bytes().to_vec())
        .collect();
    let g1_points_flat: Vec<u8> = raw_g1_points.concat();
    let aggregated_pubkey = deps.api.bls12_381_aggregate_g1(&g1_points_flat)?;

    // Aggregate signatures (G2 points)
    let raw_g2_points: Vec<_> = auth_req
        .signature_data
        .signatures
        .into_iter()
        .map(|a| a.clone().to_vec())
        .collect();
    let g2_points_flat: Vec<u8> = raw_g2_points.concat();
    let aggregated_signature = deps.api.bls12_381_aggregate_g2(&g2_points_flat)?;

    // Extract parameters
    let params: BlsMetadata = from_json(
        auth_req
            .authenticator_params
            .expect("authenticator params missing"),
    )?;

    let dst: Vec<_> = params
        .wavs_operator_avs_keys
        .iter()
        .map(|w| w.to_vec())
        .collect();

    // Hash the message to G2
    let message = auth_req.sign_mode_tx_data.sign_mode_direct;
    let hashed_message =
        deps.api
            .bls12_381_hash_to_g2(HashFunction::Sha256, &message, &dst.concat())?;

    // Verify the signature using pairing equality: e(g1, signature) == e(pubkey, H(message))
    let is_valid = deps.api.bls12_381_pairing_equality(
        &aggregated_pubkey,
        &aggregated_signature,
        &aggregated_pubkey,
        &hashed_message,
    )?;

    if !is_valid {
        return Err(ContractError::VerificationError(
            cosmwasm_std::VerificationError::GenericErr,
        ));
    }

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
