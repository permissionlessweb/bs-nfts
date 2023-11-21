use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MsgCreateNftCurveSale, MsgCreateNftSimpleSale, QueryMsg,
};
use crate::state::{Config, CONFIG};

use cosmos_sdk_proto::{cosmos::distribution::v1beta1::MsgFundCommunityPool, traits::Message};

use bs721_curve::msg::InstantiateMsg as Bs721CurveMsgInstantiate;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::must_pay;
use launchparty_fixed::msg::InstantiateMsg as LaunchpartyFixedMsgInstantiate;

const CONTRACT_NAME: &str = "crates.io:bs721-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    msg.validate(env.clone())?;

    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        bs721_metadata_code_id: msg.bs721_metadata_code_id,
        bs721_royalties_code_id: msg.bs721_royalties_code_id,
        bs721_simple_sale_code_id: msg.bs721_simple_sale_code_id,
        bs721_curve_sale_code_id: msg.bs721_curve_sale_code_id,
        protocol_fee_bps: msg.protocol_fee_bps,
        create_nft_sale_fee: msg.create_nft_sale_fee,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            bs721_metadata_code_id,
            bs721_royalties_code_id,
            protocol_fee_bps,
            bs721_simple_sale_code_id,
            bs721_curve_sale_code_id,
            create_nft_sale_fee,
        } => execute_update_config(
            deps,
            info,
            owner,
            bs721_metadata_code_id,
            bs721_royalties_code_id,
            bs721_simple_sale_code_id,
            bs721_curve_sale_code_id,
            protocol_fee_bps,
            create_nft_sale_fee,
        ),
        ExecuteMsg::CreateNftSimpleSale(msg) => {
            execute_create_nft_simple_sale(deps, env, info, msg)
        }
        ExecuteMsg::CreateNftCurveSale(msg) => execute_create_nft_curve_sale(deps, env, info, msg),
    }
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    bs721_metadata_code_id: Option<u64>,
    bs721_royalties_code_id: Option<u64>,
    bs721_simple_sale_code_id: Option<u64>,
    bs721_curve_sale_code_id: Option<u64>,
    protocol_fee_bps: Option<u32>,
    create_nft_sale_fee: Option<Coin>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    if let Some(bs721_metadata_code_id) = bs721_metadata_code_id {
        config.bs721_metadata_code_id = bs721_metadata_code_id;
    }

    if let Some(bs721_royalties_code_id) = bs721_royalties_code_id {
        config.bs721_royalties_code_id = bs721_royalties_code_id;
    }

    if let Some(bs721_simple_sale_code_id) = bs721_simple_sale_code_id {
        config.bs721_simple_sale_code_id = bs721_simple_sale_code_id;
    }

    if let Some(bs721_curve_sale_code_id) = bs721_curve_sale_code_id {
        config.bs721_curve_sale_code_id = bs721_curve_sale_code_id;
    }

    if let Some(protocol_fee_bps) = protocol_fee_bps {
        config.protocol_fee_bps = protocol_fee_bps;
    }

    if let Some(create_nft_sale_fee) = create_nft_sale_fee {
        config.create_nft_sale_fee = create_nft_sale_fee;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

fn execute_create_nft_curve_sale(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MsgCreateNftCurveSale,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let sent_amount = must_pay(&info, &config.create_nft_sale_fee.denom)?;

    if sent_amount != config.create_nft_sale_fee.amount {
        return Err(ContractError::InvalidPaymentAmount(
            sent_amount,
            config.create_nft_sale_fee.amount,
        ));
    }

    let mut res = Response::new();

    // Protocol fee
    res = res.add_submessage(fund_community_pool_msg(
        env,
        coin(
            config.create_nft_sale_fee.amount.u128(),
            config.create_nft_sale_fee.denom,
        ),
    ));

    let msg = WasmMsg::Instantiate {
        admin: Some(config.owner.to_string()),
        code_id: config.bs721_curve_sale_code_id,
        msg: to_binary(&Bs721CurveMsgInstantiate {
            symbol: msg.symbol,
            payment_denom: msg.payment_denom,
            max_per_address: msg.max_per_address,
            collection_image: msg.collection_image,
            collection_cover_image: msg.collection_cover_image,
            metadata: msg.metadata,
            seller_fee_bps: msg.seller_fee_bps,
            referral_fee_bps: msg.referral_fee_bps,
            protocol_fee_bps: config.protocol_fee_bps as u16,
            contributors: msg.contributors,
            start_time: msg.start_time,
            max_edition: msg.max_edition,
            ratio: msg.ratio,
            bs721_metadata_code_id: config.bs721_metadata_code_id,
            bs721_royalties_code_id: config.bs721_royalties_code_id,
        })?,
        funds: vec![],
        label: format!("Bs721CurveSale-{}", config.bs721_curve_sale_code_id,),
    };

    Ok(res
        .add_attribute("action", "create_nft_curve_sale")
        .add_message(msg))
}

fn execute_create_nft_simple_sale(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MsgCreateNftSimpleSale,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let sent_amount = must_pay(&info, &config.create_nft_sale_fee.denom)?;

    if sent_amount != config.create_nft_sale_fee.amount {
        return Err(ContractError::InvalidPaymentAmount(
            sent_amount,
            config.create_nft_sale_fee.amount,
        ));
    }

    let mut res = Response::new();

    // Protocol fee
    res = res.add_submessage(fund_community_pool_msg(
        env,
        coin(
            config.create_nft_sale_fee.amount.u128(),
            config.create_nft_sale_fee.denom,
        ),
    ));

    let msg = WasmMsg::Instantiate {
        admin: Some(config.owner.to_string()),
        code_id: config.bs721_simple_sale_code_id,
        msg: to_binary(&LaunchpartyFixedMsgInstantiate {
            symbol: msg.symbol,
            price: msg.price,
            max_per_address: msg.max_per_address,
            collection_image: msg.collection_image,
            collection_cover_image: msg.collection_cover_image,
            metadata: msg.metadata,
            seller_fee_bps: msg.seller_fee_bps,
            referral_fee_bps: msg.referral_fee_bps,
            protocol_fee_bps: config.protocol_fee_bps as u16,
            contributors: msg.contributors,
            start_time: msg.start_time,
            party_type: msg.party_type,
            bs721_metadata_code_id: config.bs721_metadata_code_id,
            bs721_royalties_code_id: config.bs721_royalties_code_id,
        })?,
        funds: vec![],
        label: format!("Bs721SimpleSale-{}", config.bs721_simple_sale_code_id),
    };

    Ok(res
        .add_attribute("action", "create_nft_simple_sale")
        .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

fn fund_community_pool_msg(env: Env, amount: Coin) -> SubMsg {
    let mut buffer = vec![];

    MsgFundCommunityPool {
        amount: vec![cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
            denom: amount.denom,
            amount: amount.amount.to_string(),
        }],
        depositor: env.contract.address.to_string(),
    }
    .encode(&mut buffer)
    .unwrap();

    SubMsg::new(CosmosMsg::Stargate {
        type_url: "/cosmos.distribution.v1beta1.MsgFundCommunityPool".to_string(),
        value: Binary::from(buffer),
    })
}
