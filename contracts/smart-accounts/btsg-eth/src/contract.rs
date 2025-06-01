use btsg_auth::{
    AuthenticationRequest, ConfirmExecutionRequest, OnAuthenticatorAddedRequest,
    OnAuthenticatorRemovedRequest, TrackRequest,
};
use cosmwasm_std::{
    from_json, to_json_binary, Binary, Deps, DepsMut, Env, HashFunction, MessageInfo, Response,
    StdResult, BLS12_381_G1_GENERATOR,
};
use cw2::set_contract_version;
use saa_common::Verifiable;

use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg},
    state::PAYLOAD,
    ContractError,
};

use cosmwasm_std::entry_point;

use saa::{
    msgs::MsgDataToSign,
    types::{ClientData, ClientDataOtherKeys},
    utils::passkey::base64_to_url,
    EthPersonalSign, PasskeyCredential,
};

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
    let cred = EthPersonalSign {
        message: auth_req.sign_mode_tx_data.sign_mode_direct,
        signature: auth_req.signature,
        signer: auth_req.signature_data.signers[0].to_string(),
    };

    // assert client origin is same as one registered

    // verify ethereum personal signature 
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

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::mock_dependencies;
    use saa::EthPersonalSign;
    use saa_common::Verifiable;

    #[test]
    fn eth_personal_verifiable() {
        let deps = mock_dependencies();

        let message = r#"{"chain_id":"elgafar-1","contract_address":"stars1gjgfp9wps9c0r3uqhr0xxfgu02rnzcy6gngvwpm7a78j7ykfqquqr2fuj4","messages":["Create TBA account"],"nonce":"0"}"#;
        let address = "0xac03048da6065e584d52007e22c69174cdf2b91a";
        let base = "eyJjaGFpbl9pZCI6ImVsZ2FmYXItMSIsImNvbnRyYWN0X2FkZHJlc3MiOiJzdGFyczFnamdmcDl3cHM5YzByM3VxaHIweHhmZ3UwMnJuemN5NmduZ3Z3cG03YTc4ajd5a2ZxcXVxcjJmdWo0IiwibWVzc2FnZXMiOlsiQ3JlYXRlIFRCQSBhY2NvdW50Il0sIm5vbmNlIjoiMCJ9";
        let message = cosmwasm_std::Binary::new(message.as_bytes().to_vec());
        assert!(message.to_base64() == base, "not euqal");

        let signature = cosmwasm_std::Binary::from_base64(
            "a/lQuaTyhcTEeRA2XFTPxoDSIdS3yUUH1VSKOm2zz5EURfheGzzLgXea6QAalswOM2njnUzblqIGiOC0P+j2rhw="
        ).unwrap();

        let cred = EthPersonalSign {
            signer: address.to_string(),
            signature: signature.clone(),
            message,
        };
        let res = cred.verify_cosmwasm(deps.as_ref().api);
        println!("Res: {:?}", res);
        assert!(res.is_ok())
    }
}
