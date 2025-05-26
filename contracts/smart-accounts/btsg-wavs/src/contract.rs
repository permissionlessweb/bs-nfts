use btsg_auth::{
    AuthenticationRequest, ConfirmExecutionRequest, OnAuthenticatorAddedRequest,
    OnAuthenticatorRemovedRequest, TrackRequest,
};
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, HashFunction, MessageInfo, Response, StdResult,
    BLS12_381_G1_GENERATOR,
};
use cw2::set_contract_version;

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg},
    state::WAVS_PUBKEY,
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
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.wavs_operator_pubkeys.len() > 10 {
        return Err(ContractError::TooManyWavsKeys {});
    }

    WAVS_PUBKEY.save(deps.storage, &msg.wavs_operator_pubkeys)?;
    cw_ownable::initialize_owner(
        deps.storage,
        deps.api,
        Some(msg.owner.unwrap_or(info.sender).as_str()),
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
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
    let pubkeys = WAVS_PUBKEY.load(deps.storage)?;
    // assert the wavs operator signature length
    let a = auth_req.signature_data.signers.len();
    let b = pubkeys.len();
    if a != b {
        return Err(ContractError::InvalidPubkeyCount { a, b });
    }
    // EXAMPLE IMPLEMENTATION FOR BLS12_381 VERIFICATION
    let dst = b"QUUX-V01-CS02-with-BLS12381G1_XMD:SHA-256_SSWU_RO_";
    // Aggregate public keys when registered (G1 points)
    let wavs_ops_pubkeys: Vec<_> = pubkeys.iter().map(|a| a.to_vec()).collect();

    // Aggregate signatures (G2 points)
    let wavs_ops_signatures: Vec<_> = auth_req
        .signature_data
        .signatures
        .into_iter()
        .map(|a| a.clone().to_vec())
        .collect();

    let aggregated_signature = deps
        .api
        .bls12_381_aggregate_g2(&wavs_ops_signatures.concat())?;

    // Aggregate the pubkey (G1 points)
    let aggregated_pubkey = deps
        .api
        .bls12_381_aggregate_g1(&wavs_ops_pubkeys.concat())?;

    // hash the json encoded Any (Stargate) msg ,into g2 (signature)
    let hashed_message = deps.api.bls12_381_hash_to_g2(
        HashFunction::Sha256,
        &to_json_binary(&auth_req.msg)?,
        dst,
    )?;

    // Verify the signature using pairing equality: e(g1, signature) == e(pubkey, H(message))
    if !deps.api.bls12_381_pairing_equality(
        &BLS12_381_G1_GENERATOR,
        &aggregated_signature,
        &aggregated_pubkey,
        &hashed_message,
    )? {
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

pub fn execute_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    action: cw_ownable::Action,
) -> Result<Response, ContractError> {
    let ownership = cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
    Ok(Response::default().add_attributes(ownership.into_attributes()))
}
