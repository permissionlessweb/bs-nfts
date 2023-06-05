use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

use bs721_base::{Extension, MintMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    has_coins, to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
    Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;

use bs721_base::msg::ExecuteMsg as Bs721ExecuteMsg;
use cw_utils::parse_reply_instantiate_data;

const CONTRACT_NAME: &str = "crates.io:bs721-drop-simple";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_TOKEN_REPLY_ID: u64 = 1;
const INSTANTIATE_ROYALTY_REPLY_ID: u64 = 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    msg.clone().validate()?;

    let config: Config = Config {
        name: msg.name.clone(),
        symbol: msg.symbol.clone(),
        base_token_uri: msg.base_token_uri.clone(),
        price: msg.price.clone(),
        max_editions: msg.max_editions,
        royalty_address: None,
        seller_fee_bps: msg.seller_fee_bps,
        referral_fee_bps: msg.referral_fee_bps,
        start_time: msg.start_time,
        duration: msg.duration,
        bs721_address: None,
        next_token_id: 1,
        creator: info.sender.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    let bs721_msg = msg
        .clone()
        .into_bs721_wasm_msg(env.contract.address.to_string());
    let bs721_msg: SubMsg<Empty> = SubMsg::reply_on_success(bs721_msg, INSTANTIATE_TOKEN_REPLY_ID);

    let cw4_msg = msg.into_bs721_royalty_wasm_msg();
    let cw4_msg: SubMsg<Empty> = SubMsg::reply_on_success(cw4_msg, INSTANTIATE_ROYALTY_REPLY_ID);

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("sender", info.sender)
        .add_submessage(bs721_msg)
        .add_submessage(cw4_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    match reply.id {
        INSTANTIATE_TOKEN_REPLY_ID => {
            let res = parse_reply_instantiate_data(reply).unwrap();
            if config.bs721_address.is_some() {
                return Err(ContractError::Bs721AlreadyLinked {});
            }

            let bs721_address = deps.api.addr_validate(&res.contract_address)?;
            config.bs721_address = Some(bs721_address);
            CONFIG.save(deps.storage, &config)?;

            Ok(Response::default()
                .add_attribute("action", "bs721_reply")
                .add_attribute("contract_address", res.contract_address))
        }
        INSTANTIATE_ROYALTY_REPLY_ID => {
            let res = parse_reply_instantiate_data(reply).unwrap();
            if config.royalty_address.is_some() {
                return Err(ContractError::RoyaltyContractAlreadyLinked {});
            }

            let royalty_address = deps.api.addr_validate(&res.contract_address)?;
            config.royalty_address = Some(royalty_address);
            CONFIG.save(deps.storage, &config)?;

            Ok(Response::default()
                .add_attribute("action", "cw4_reply")
                .add_attribute("contract_address", res.contract_address))
        }
        _ => Err(ContractError::UnknownReplyId {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint { amount, referral } => execute_mint(deps, env, info, amount, referral),
    }
}

fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<u32>,
    referral: Option<String>,
) -> Result<Response, ContractError> {
    // if amount is None, default to 1
    let amt_to_mint = amount.unwrap_or(1);

    // validate referral address
    if referral.clone().is_some() {
        deps.api.addr_validate(&referral.clone().unwrap())?;
    }

    // load config
    let mut config: Config = CONFIG.load(deps.storage)?;

    // check if the sale is active
    if config.start_time.clone() > env.block.time {
        return Err(ContractError::SaleNotStarted {});
    }

    // check if bs721 contract is linked
    if config.bs721_address.clone().is_none() {
        return Err(ContractError::Bs721NotLinked {});
    }

    let bs721_address = config.bs721_address.clone().unwrap();

    // check if amount is available
    if (config.next_token_id - 1) + amt_to_mint > config.max_editions {
        return Err(ContractError::NotEnoughTokens {});
    }

    // check if the funds are enough
    let price = config.price.clone();
    if price.amount > Uint128::from(0u128)
        && !has_coins(
            &info.funds,
            &Coin {
                denom: price.denom.clone(),
                amount: price
                    .amount
                    .checked_mul(Uint128::from(amt_to_mint as u128))
                    .unwrap(),
            },
        )
    {
        return Err(ContractError::NotEnoughFunds {});
    }

    // mint the tokens
    let mut res = Response::new();
    let sender = info.sender.clone();

    for _ in 0..amt_to_mint {
        let token_id = config.next_token_id.clone();
        let token_uri = format!("{}{}", config.base_token_uri.clone(), token_id);

        let mint_msg = WasmMsg::Execute {
            contract_addr: bs721_address.to_string(),
            msg: to_binary(&Bs721ExecuteMsg::<Extension, Empty>::Mint(MintMsg::<
                Extension,
            > {
                owner: sender.to_string(),
                token_id: token_id.to_string(),
                token_uri: Some(token_uri),
                extension: None,
                payment_addr: Some(config.royalty_address.clone().unwrap().to_string()),
                seller_fee_bps: Some(config.seller_fee_bps.clone()),
            }))?,
            funds: vec![],
        };

        res = res
            .add_message(mint_msg)
            .add_attribute("action", "mint")
            .add_attribute("token_id", token_id.to_string())
            .add_attribute("recipient", sender.to_string());

        config.next_token_id += 1;
        CONFIG.save(deps.storage, &config)?;
    }

    // if price is > 0
    // and if referral fee is > 0
    // and if referral is not None
    // then send the fee to the referral
    if price.amount > Uint128::from(0u128) && config.referral_fee_bps > 0 && referral.is_some() {
        let referral_fee = price
            .amount
            .checked_mul(Uint128::from(config.referral_fee_bps as u128))
            .unwrap()
            .checked_div(Uint128::from(10000u128))
            .unwrap();

        let referral_fee = Coin {
            denom: price.denom.clone(),
            amount: referral_fee
                .checked_mul(Uint128::from(amt_to_mint as u128))
                .unwrap(),
        };

        res = res
            .add_message(BankMsg::Send {
                to_address: referral.clone().unwrap(),
                amount: vec![referral_fee.clone()],
            })
            .add_attribute("referral", referral.unwrap())
            .add_attribute("fee", referral_fee.to_string());
    }

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_get_config(deps)?),
    }
}

pub fn query_get_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        max_editions: config.max_editions,
        next_token_id: config.next_token_id,
        start_time: config.start_time,
        base_token_uri: config.base_token_uri,
        price: config.price,
        referral_fee_bps: config.referral_fee_bps,
        seller_fee_bps: config.seller_fee_bps,
        bs721_address: config.bs721_address,
        royalty_address: config.royalty_address,
        creator: config.creator,
        name: config.name,
        symbol: config.symbol,
        duration: config.duration,
    })
}
