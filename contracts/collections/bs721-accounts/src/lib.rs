pub mod commands;
pub mod error;
pub mod msg;
pub mod state;
pub mod sudo;

use std::marker::PhantomData;

pub use error::ContractError;

use crate::{
    msg::{InstantiateMsg, SudoParams},
    state::{ACCOUNT_MARKETPLACE, SUDO_PARAMS, VERIFIER},
};
use semver::Version;

use crate::commands::*;
use bs721_base::{ContractError as Bs721ContractError, MintMsg};
use bs_account::Metadata;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult,
};
use cw_utils::maybe_addr;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CONTRACT_NAME: &str = "bs721-accounts";

pub type Bs721AccountsContract<'a> = bs721_base::Bs721Contract<'a, Metadata, Empty, Empty, Empty>;
pub type ExecuteMsg = crate::msg::ExecuteMsg<Metadata>;
pub type QueryMsg = crate::msg::QueryMsg;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Bs721ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Initialize max record count to 10, can be changed by sudo params
    SUDO_PARAMS.save(
        deps.storage,
        &SudoParams {
            max_record_count: 10,
        },
    )?;

    let api = deps.api;
    VERIFIER.set(deps.branch(), maybe_addr(api, msg.verifier)?)?;
    ACCOUNT_MARKETPLACE.save(deps.storage, &msg.marketplace)?;

    let res =
        Bs721AccountsContract::default().instantiate(deps, env.clone(), info, msg.base_init_msg)?;

    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("bs721_account_address", env.contract.address.to_string()))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::AssociateAddress { name, address } => {
            execute_associate_address(deps, info, name, address)
        }
        ExecuteMsg::UpdateImageNft { name, nft } => execute_update_image_nft(deps, info, name, nft),
        ExecuteMsg::AddTextRecord { name, record } => {
            execute_add_text_record(deps, info, name, record)
        }
        ExecuteMsg::RemoveTextRecord { name, record_name } => {
            execute_remove_text_record(deps, info, name, record_name)
        }
        ExecuteMsg::UpdateTextRecord { name, record } => {
            execute_update_text_record(deps, info, name, record)
        }
        ExecuteMsg::VerifyTextRecord {
            name,
            record_name,
            result,
        } => execute_verify_text_record(deps, info, name, record_name, result),
        ExecuteMsg::UpdateVerifier { verifier } => {
            Ok(VERIFIER.execute_update_admin(deps, info, maybe_addr(api, verifier)?)?)
        }
        ExecuteMsg::SetMarketplace { address } => {
            execute_set_profile_marketplace(deps, info, address)
        }
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => execute_transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => execute_send_nft(deps, env, info, contract, token_id, msg),
        ExecuteMsg::Mint(MintMsg {
            token_id,
            owner,
            token_uri,
            seller_fee_bps,
            payment_addr,
            extension,
        }) => execute_mint(
            deps,
            info,
            bs721_base::ExecuteMsg::Mint {
                token_id,
                owner,
                token_uri,
                extension,
                seller_fee_bps,
                payment_addr,
            },
        ),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
        _ => Bs721AccountsContract::default()
            .execute(deps, env, info, msg.into())
            .map_err(|e| e.into()),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
        QueryMsg::AccountMarketplace {} => to_json_binary(&query_profile_marketplace(deps)?),
        QueryMsg::Account { address } => to_json_binary(&query_name(deps, address)?),
        QueryMsg::Verifier {} => to_json_binary(&VERIFIER.query_admin(deps)?),
        QueryMsg::AssociatedAddress { name } => {
            to_json_binary(&query_associated_address(deps, &name)?)
        }
        QueryMsg::ImageNFT { name } => to_json_binary(&query_image_nft(deps, &name)?),
        QueryMsg::TextRecords { name } => to_json_binary(&query_text_records(deps, &name)?),
        QueryMsg::IsTwitterVerified { name } => {
            to_json_binary(&query_is_twitter_verified(deps, &name)?)
        }
        _ => Bs721AccountsContract::default().query(deps, env, msg.into()),
    }
}
