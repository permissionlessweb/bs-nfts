use crate::error::ContractError;
use crate::{ExecuteMsg, InstantiateMsg, MintTicketObject, QueryMsg};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_json_binary, AnyMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::{Item, Map};

// Storage
pub const WAVS_SMART_ACCOUNT: Item<String> = Item::new("wsa");
pub const FANTOKEN_INFO: Item<FantokenInfo> = Item::new("fantoken_info");
pub const MINTED_AMOUNTS: Map<&str, Uint128> = Map::new("minted_amounts");

// Constants
const CONTRACT_NAME: &str = "crates.io:cw-fantoken-infuser";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const FANTOKEN_CREATE_REPLY_ID: u64 = 1;
const FANTOKEN_MINT_REPLY_ID: u64 = 2;

// Message types for the fantoken module
#[cw_serde]
pub struct CreateFantokenMsg {
    pub symbol: String,
    pub name: String,
    pub max_supply: String, // Using String to handle sdk.Int serialization
    pub authority: String,
    pub uri: String,
    pub minter: String,
}

#[cw_serde]
pub struct MintFantokenMsg {
    pub recipient: String,
    pub coin: Coin,
    pub minter: String,
}

#[cw_serde]
pub struct SetFantokenUriMsg {
    pub denom: String,
    pub uri: String,
    pub authority: String,
}

#[cw_serde]
pub struct FantokenInfo {
    pub symbol: String,
    pub name: String,
    pub max_supply: Uint128,
    pub authority: String,
    pub uri: String,
    pub minter: String,
    pub denom: String, // Will be set after creation
}

fn handle_mint_fantokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mint_requests: Vec<MintTicketObject>,
) -> Result<Response, ContractError> {
    // Ensure only the smart account can mint fantokens
    if info.sender.to_string() != WAVS_SMART_ACCOUNT.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }

    let fantoken_info = FANTOKEN_INFO.load(deps.storage)?;

    if fantoken_info.denom.is_empty() {
        return Err(ContractError::FantokenNotCreated {});
    }

    let mut mint_msgs = Vec::new();
    let mut total_to_mint = Uint128::zero();

    for mint_request in mint_requests {
        let amount = Uint128::from(mint_request.amount);
        total_to_mint += amount;

        let mint_msg = AnyMsg {
            type_url: "/bitsong.fantoken.v1beta1.MsgMint".into(),
            value: to_json_binary(&MintFantokenMsg {
                recipient: mint_request.ticket, // recipient address
                coin: coin(amount.u128(), fantoken_info.denom.clone()),
                minter: env.contract.address.to_string(),
            })?,
        };

        mint_msgs.push(mint_msg);
    }

    // Check if minting would exceed max supply
    let current_minted = MINTED_AMOUNTS
        .may_load(deps.storage, &fantoken_info.denom)?
        .unwrap_or_default();

    if current_minted + total_to_mint > fantoken_info.max_supply {
        return Err(ContractError::ExceedsMaxSupply {});
    }

    // Update minted amount
    MINTED_AMOUNTS.save(
        deps.storage,
        &fantoken_info.denom,
        &(current_minted + total_to_mint),
    )?;

    Ok(Response::new().add_messages(mint_msgs))
}

fn handle_set_uri(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_uri: String,
) -> Result<Response, ContractError> {
    // Ensure only the smart account can update URI
    if info.sender.to_string() != WAVS_SMART_ACCOUNT.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }

    let mut fantoken_info = FANTOKEN_INFO.load(deps.storage)?;

    if fantoken_info.denom.is_empty() {
        return Err(ContractError::FantokenNotCreated {});
    }

    // Update stored URI
    fantoken_info.uri = new_uri.clone();
    FANTOKEN_INFO.save(deps.storage, &fantoken_info)?;

    let set_uri_msg = AnyMsg {
        type_url: "/bitsong.fantoken.v1beta1.MsgSetUri".into(),
        value: to_json_binary(&SetFantokenUriMsg {
            denom: fantoken_info.denom,
            uri: new_uri,
            authority: env.contract.address.to_string(),
        })?,
    };

    Ok(Response::new().add_message(set_uri_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    InstantiateMsg {
        symbol,
        name,
        max_supply,
        uri,
    }: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    WAVS_SMART_ACCOUNT.save(deps.storage, &info.sender.to_string())?;

    // Store fantoken info (denom will be updated in reply)
    let fantoken_info = FantokenInfo {
        symbol: symbol.clone(),
        name: name.clone(),
        max_supply,
        authority: env.contract.address.to_string(),
        uri: uri.clone(),
        minter: env.contract.address.to_string(),
        denom: String::new(), // Will be set in reply
    };

    FANTOKEN_INFO.save(deps.storage, &fantoken_info)?;

    // Create the fantoken creation message
    let create_msg = AnyMsg {
        type_url: "/bitsong.fantoken.v1beta1.MsgIssue".into(),
        value: to_json_binary(&CreateFantokenMsg {
            symbol,
            name,
            max_supply: max_supply.to_string(),
            authority: env.contract.address.to_string(),
            uri,
            minter: env.contract.address.to_string(),
        })?,
    };

    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(
            create_msg,
            FANTOKEN_CREATE_REPLY_ID,
        ))
        .add_attribute("method", "instantiate")
        .add_attribute("smart_account", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MintFantokens { data } => handle_mint_fantokens(deps, env, info, data),
        ExecuteMsg::SetUri { uri } => handle_set_uri(deps, env, info, uri),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetFantokenInfo {} => {
            let info = FANTOKEN_INFO.may_load(deps.storage)?;
            to_json_binary(&info)
        }
        QueryMsg::GetMintedAmount {} => {
            let fantoken_info = FANTOKEN_INFO.load(deps.storage)?;
            let minted = MINTED_AMOUNTS
                .may_load(deps.storage, &fantoken_info.denom)?
                .unwrap_or_default();
            to_json_binary(&minted)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        FANTOKEN_CREATE_REPLY_ID => {
            // Handle successful fantoken creation
            // The denom should be in the response, but for now we'll construct it
            // In a real implementation, you'd parse the response to get the actual denom
            let mut fantoken_info = FANTOKEN_INFO.load(deps.storage)?;

            // Construct denom based on typical fantoken module pattern
            // This might need adjustment based on your specific fantoken module implementation
            fantoken_info.denom = format!("ft{}", fantoken_info.symbol.to_lowercase());

            FANTOKEN_INFO.save(deps.storage, &fantoken_info)?;

            Ok(Response::new()
                .add_attribute("method", "fantoken_created")
                .add_attribute("denom", &fantoken_info.denom))
        }
        _ => Err(ContractError::UnknownReplyId {}),
    }
}
