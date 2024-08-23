use std::vec;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_json_binary, Addr, Binary, Deps, DepsMut, Empty, Env, Event, MessageInfo, Reply,
    Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::{maybe_addr, must_pay, parse_reply_instantiate_data};

use semver::Version;

use btsg_account::{
    account::{
        ExecuteMsg as BsAccountExecuteMsg, InstantiateMsg as BsAccountCollectionInstantiateMsg,
    },
    common::{charge_fees, SECONDS_PER_YEAR},
    market::ExecuteMsg as MarketplaceExecuteMsg,
    minter::{Config, SudoParams},
    Metadata,
};

use crate::commands::{
    execute_mint_and_list, execute_pause, execute_update_config, query_collection, query_config,
    query_params,
};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::state::{
    WhitelistContract, WhitelistContractType, ACCOUNT_COLLECTION, ACCOUNT_MARKETPLACE, ADMIN,
    CONFIG, PAUSED, SUDO_PARAMS,
};
use crate::sudo::*;

// // version info for migration info
const CONTRACT_NAME: &str = "crates.io:account-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INIT_COLLECTION_REPLY_ID: u64 = 1;
const TRADING_START_TIME_OFFSET_IN_SECONDS: u64 = 2 * SECONDS_PER_YEAR;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin_addr = maybe_addr(deps.api, msg.admin)?;
    ADMIN.set(deps.branch(), admin_addr)?;

    PAUSED.save(deps.storage, &false)?;

    let marketplace = deps.api.addr_validate(&msg.marketplace_addr)?;
    ACCOUNT_MARKETPLACE.save(deps.storage, &marketplace)?;

    let params = SudoParams {
        min_name_length: msg.min_name_length,
        max_name_length: msg.max_name_length,
        base_price: msg.base_price,
    };
    SUDO_PARAMS.save(deps.storage, &params)?;

    let config = Config {
        public_mint_start_time: env.block.time.plus_seconds(1),
    };
    CONFIG.save(deps.storage, &config)?;

    let account_collection_init_msg = BsAccountCollectionInstantiateMsg {
        verifier: msg.verifier,
        base_init_msg: bs721_base::InstantiateMsg {
            name: "Bitsong Account Tokens".to_string(),
            symbol: "ACCOUNTS".to_string(),
            minter: env.contract.address.to_string(),
            uri: None,
        },
        marketplace,
    };

    let wasm_msg = WasmMsg::Instantiate {
        code_id: msg.collection_code_id,
        msg: to_json_binary(&account_collection_init_msg)?,
        funds: info.funds,
        admin: Some(info.sender.to_string()),
        label: "Account Collection".to_string(),
    };
    let submsg = SubMsg::reply_on_success(wasm_msg, INIT_COLLECTION_REPLY_ID);

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("account_minter_addr", env.contract.address.to_string())
        .add_submessage(submsg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::MintAndList { account } => {
            execute_mint_and_list(deps, info, env, account.trim())
        }
        ExecuteMsg::UpdateAdmin { admin } => {
            Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, admin)?)?)
        }
        ExecuteMsg::Pause { pause } => execute_pause(deps, info, pause),
        ExecuteMsg::UpdateConfig { config } => execute_update_config(deps, info, env, config),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_json_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::Collection {} => to_json_binary(&query_collection(deps)?),
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

/// Mint a name for the sender, or `contract` if specified

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INIT_COLLECTION_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            let collection_address = &res.contract_address;

            ACCOUNT_COLLECTION.save(deps.storage, &Addr::unchecked(collection_address))?;

            let msg = WasmMsg::Execute {
                contract_addr: collection_address.to_string(),
                funds: vec![],
                msg: to_json_binary(
                    &(BsAccountExecuteMsg::<Metadata>::SetMarketplace {
                        address: ACCOUNT_MARKETPLACE.load(deps.storage)?.to_string(),
                    }),
                )?,
            };

            Ok(Response::default()
                .add_attribute("action", "init_collection_reply")
                .add_message(msg))
        }
        Err(_) => Err(ContractError::ReplyOnSuccess {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    let current_version = cw2::get_contract_version(deps.storage)?;
    if current_version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Cannot upgrade to a different contract").into());
    }
    let version: Version = current_version
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;

    if version > new_version {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if version == new_version {
        return Ok(Response::new());
    }

    // set new contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        SudoMsg::UpdateParams {
            min_name_length,
            max_name_length,
            base_price,
            // fair_burn_bps,
        } => sudo_update_params(
            deps,
            min_name_length,
            max_name_length,
            base_price,
            // fair_burn_bps,
        ),
        SudoMsg::UpdateAccountCollection { collection } => {
            sudo_update_name_collection(deps, api.addr_validate(&collection)?)
        }
        SudoMsg::UpdateAccountMarketplace { marketplace } => {
            sudo_update_account_marketplace(deps, api.addr_validate(&marketplace)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Addr, MessageInfo};

    use crate::commands::{validate_name, validate_payment};

    use super::*;

    #[test]
    fn check_validate_name() {
        let min = 3;
        let max = 63;
        assert!(validate_name("bobo", min, max).is_ok());
        assert!(validate_name("-bobo", min, max).is_err());
        assert!(validate_name("bobo-", min, max).is_err());
        assert!(validate_name("bo-bo", min, max).is_ok());
        assert!(validate_name("bo--bo", min, max).is_err());
        assert!(validate_name("bob--o", min, max).is_ok());
        assert!(validate_name("bo", min, max).is_err());
        assert!(validate_name("b", min, max).is_err());
        assert!(validate_name("bob", min, max).is_ok());
        assert!(validate_name(
            "bobobobobobobobobobobobobobobobobobobobobobobobobobobobobobobo",
            min,
            max
        )
        .is_ok());
        assert!(validate_name(
            "bobobobobobobobobobobobobobobobobobobobobobobobobobobobobobobob",
            min,
            max
        )
        .is_err());
        assert!(validate_name("0123456789", min, max).is_ok());
        assert!(validate_name("😬", min, max).is_err());
        assert!(validate_name("BOBO", min, max).is_err());
        assert!(validate_name("b-o----b", min, max).is_ok());
        assert!(validate_name("bobo.stars", min, max).is_err());
    }

    #[test]
    fn check_validate_payment() {
        let base_price = 100_000_000;

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(base_price, "ubtsg")],
        };
        assert_eq!(
            validate_payment(5, &info, base_price)
                .unwrap()
                .unwrap()
                .amount
                .u128(),
            base_price
        );

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(base_price * 10, "ubtsg")],
        };
        assert_eq!(
            validate_payment(4, &info, base_price)
                .unwrap()
                .unwrap()
                .amount
                .u128(),
            base_price * 10
        );

        let info = MessageInfo {
            sender: Addr::unchecked("sender"),
            funds: vec![coin(base_price * 100, "ubtsg")],
        };
        assert_eq!(
            validate_payment(3, &info, base_price)
                .unwrap()
                .unwrap()
                .amount
                .u128(),
            base_price * 100
        );
    }

    // #[test]
    // fn check_validate_payment_with_flatrate_discount() {
    //     let base_price = 100_000_000;

    //     let info = MessageInfo {
    //         sender: Addr::unchecked("sender"),
    //         funds: vec![coin(base_price, "ubtsg")],
    //     };
    //     assert_eq!(
    //         // we treat the discount as a flat amount given as 100.0
    //         validate_payment(
    //             5, &info, base_price,
    //             // Some(contract::Discount::Flatrate(100)),
    //         )
    //         .unwrap()
    //         .unwrap()
    //         .amount
    //         .u128(),
    //         base_price
    //     );
    // }
}
