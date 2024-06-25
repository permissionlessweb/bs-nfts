use bs721_base::MintMsg;
use bs721_profile::ExecuteMsg;
use bs_profile::{
    common::charge_fees, market::ExecuteMsg as MarketplaceExecuteMsg, minter::Config, Metadata,
};
use bs_std::NATIVE_DENOM;
use cosmwasm_std::to_json_binary;
use cosmwasm_std::{
    coin, Coin, Decimal, DepsMut, Env, Event, MessageInfo, Response, Uint128, WasmMsg,
};
use cw_utils::must_pay;

use crate::state::PROFILE_MARKETPLACE;
use crate::{
    state::{ADMIN, CONFIG, PROFILE_COLLECTION, PAUSED, SUDO_PARAMS, WHITELISTS},
    ContractError,
};

pub fn execute_mint_and_list(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    name: &str,
) -> Result<Response, ContractError> {
    if PAUSED.load(deps.storage)? {
        return Err(ContractError::MintingPaused {});
    }

    let sender = &info.sender.to_string();
    let mut res = Response::new();
    // let config = CONFIG.load(deps.storage)?;    
    let params = SUDO_PARAMS.load(deps.storage)?;

    validate_name(name, params.min_name_length, params.max_name_length)?;
    
    // let whitelists = WHITELISTS.load(deps.storage)?;
    // Assumes no duplicate addresses between whitelists
    // Otherwise there will be edge cases with per addr limit between the whitelists
    // currently this is going to match the _first_ WL they appear in...
    // let list = whitelists
    //     .iter()
    //     .find(|whitelist| match whitelist.contract_type {
    //         WhitelistContractType::UpdatableFlatrateDiscount => {
    //             let contract = WhitelistUpdatableFlatrateContract(whitelist.addr.clone());
    //             contract
    //                 .includes(&deps.querier, sender.to_string())
    //                 .unwrap_or(false)
    //         }
    //         WhitelistContractType::UpdatablePercentDiscount => {
    //             let contract = WhitelistUpdatableContract(whitelist.addr.clone());
    //             contract
    //                 .includes(&deps.querier, sender.to_string())
    //                 .unwrap_or(false)
    //         }
    //     });

    // if not on any whitelist, check public mint start time
    // if env.block.time < config.public_mint_start_time {
    //     // list.is_none() &&
    //     return Err(ContractError::MintingNotStarted {});
    // }

    // if let Some(list) = list {
    //     match list.contract_type {
    //         WhitelistContractType::UpdatableFlatrateDiscount => {
    //             let contract = WhitelistUpdatableFlatrateContract(list.addr.clone());
    //             res.messages
    //                 .push(SubMsg::new(contract.process_address(sender)?));
    //         }
    //         WhitelistContractType::UpdatablePercentDiscount => {
    //             let contract = WhitelistUpdatableContract(list.addr.clone());
    //             res.messages
    //                 .push(SubMsg::new(contract.process_address(sender)?));
    //         }
    //     }
    // }

    // let discount = list.map(|list| match list.contract_type {
    //     WhitelistContractType::UpdatableFlatrateDiscount => {
    //         let contract = WhitelistUpdatableFlatrateContract(list.addr.clone());
    //         Discount::Flatrate(
    //             contract
    //                 .mint_discount_amount(&deps.querier)
    //                 .unwrap()
    //                 .unwrap_or(0u64),
    //         )
    //     }
    // WhitelistContractType::UpdatablePercentDiscount => {
    //     let contract = WhitelistUpdatableContract(list.addr.clone());
    //     Discount::Percent(
    //         contract
    //             .mint_discount_percent(&deps.querier)
    //             .unwrap()
    //             .unwrap_or(Decimal::zero()),
    //     )
    // }
    // });

    let price = validate_payment(name.len(), &info, params.base_price.u128())?;
    if price.is_some() {
        charge_fees(
            &mut res,
            price.clone().unwrap().amount,
            // params.fair_burn_percent,
        );
    }

    let marketplace = PROFILE_MARKETPLACE.load(deps.storage)?;
    let collection = PROFILE_COLLECTION.load(deps.storage)?;
    
    let mint_msg = ExecuteMsg::Mint(MintMsg::<Metadata> {
        token_id: name.to_string(),
        owner: sender.to_string(),
        token_uri: None,
        extension: Metadata::default(),
        seller_fee_bps: None,
        payment_addr: None,
    });
    let mint_msg_exec = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_json_binary(&mint_msg)?,
        funds: vec![],
    };

    let ask_msg = MarketplaceExecuteMsg::SetAsk {
        token_id: name.to_string(),
        seller: sender.to_string(),
    };
    let list_msg_exec = WasmMsg::Execute {
        contract_addr: marketplace.to_string(),
        msg: to_json_binary(&ask_msg)?,
        funds: vec![],
    };

    let event = Event::new("mint-and-list")
        .add_attribute("name", name)
        .add_attribute("owner", sender)
        .add_attribute(
            "price",
            price
                .unwrap_or_else(|| coin(0, NATIVE_DENOM))
                .amount
                .to_string(),
        );
    Ok(res
        .add_event(event)
        .add_message(mint_msg_exec)
        .add_message(list_msg_exec))
}

/// Pause or unpause minting
pub fn execute_pause(
    deps: DepsMut,
    info: MessageInfo,
    pause: bool,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    PAUSED.save(deps.storage, &pause)?;

    let event = Event::new("pause").add_attribute("pause", pause.to_string());
    Ok(Response::new().add_event(event))
}

// pub fn execute_add_whitelist(
//     deps: DepsMut,
//     info: MessageInfo,
//     address: String,
//     whitelist_type: String,
// ) -> Result<Response, ContractError> {
//     ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

//     let whitelist = deps
//         .api
//         .addr_validate(&address)
//         .map(WhitelistUpdatableFlatrateContract)?;

//     let mut lists = WHITELISTS.load(deps.storage)?;
//     match whitelist_type.as_str() {
//         "FlatrateDiscount" => {
//             lists.push(WhitelistContract {
//                 contract_type: WhitelistContractType::UpdatableFlatrateDiscount,
//                 addr: whitelist.addr(),
//             });
//         }
//         "PercentDiscount" => {
//             lists.push(WhitelistContract {
//                 contract_type: WhitelistContractType::UpdatablePercentDiscount,
//                 addr: whitelist.addr(),
//             });
//         }
//         _ => return Err(ContractError::InvalidWhitelistType {}),
//     }

//     WHITELISTS.save(deps.storage, &lists)?;

//     let event = Event::new("add-whitelist").add_attribute("address", address);
//     Ok(Response::new().add_event(event))
// }

// pub fn execute_remove_whitelist(
//     deps: DepsMut,
//     info: MessageInfo,
//     address: String,
// ) -> Result<Response, ContractError> {
//     ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

//     let whitelist = deps.api.addr_validate(&address)?;
//     let mut lists = WHITELISTS.load(deps.storage)?;
//     lists.retain(|whitelist_contract| whitelist_contract.addr != whitelist);

//     WHITELISTS.save(deps.storage, &lists)?;

//     let event = Event::new("remove-whitelist").add_attribute("address", address);
//     Ok(Response::new().add_event(event))
// }

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    config: Config,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let start_time = config.public_mint_start_time;

    // Can not set public mint time in the past
    if env.block.time > start_time {
        return Err(ContractError::InvalidTradingStartTime(
            env.block.time,
            start_time,
        ));
    }

    CONFIG.save(deps.storage, &config)?;

    let event = Event::new("update-config").add_attribute("address", info.sender.to_string());
    Ok(Response::new().add_event(event))
}

// This follows the same rules as Internet domain names
pub fn validate_name(name: &str, min: u32, max: u32) -> Result<(), ContractError> {
    let len = name.len() as u32;
    if len < min {
        return Err(ContractError::NameTooShort {});
    } else if len >= max {
        return Err(ContractError::NameTooLong {});
    }

    name.find(invalid_char)
        .map_or(Ok(()), |_| Err(ContractError::InvalidName {}))?;

    (if name.starts_with('-') || name.ends_with('-') {
        Err(ContractError::InvalidName {})
    } else {
        Ok(())
    })?;

    if len > 4 && name[2..4].contains("--") {
        return Err(ContractError::InvalidName {});
    }

    Ok(())
}

pub enum Discount {
    Flatrate(u64),
    Percent(Decimal),
}

pub fn validate_payment(
    name_len: usize,
    info: &MessageInfo,
    base_price: u128,
    // discount: Option<Discount>,
) -> Result<Option<Coin>, ContractError> {
    // Because we know we are left with ASCII chars, a simple byte count is enough
    let amount: Uint128 = (match name_len {
        0..=2 => {
            return Err(ContractError::NameTooShort {});
        }
        3 => base_price * 100,
        4 => base_price * 10,
        _ => base_price,
    })
    .into();

    // match discount {
    //     Some(Discount::Flatrate(discount)) => {
    //         let discount = Uint128::from(discount);
    //         if amount.ge(&discount) {
    //             amount = amount
    //                 .checked_sub(discount)
    //                 .map_err(|_| StdError::generic_err("invalid discount amount"))?;
    //         }
    //     }
    //     Some(Discount::Percent(discount)) => {
    //         amount = amount * (Decimal::one() - discount);
    //     }
    //     None => {}
    // }

    if amount.is_zero() {
        return Ok(None);
    }

    let payment = must_pay(info, NATIVE_DENOM)?;
    if payment != amount {
        return Err(ContractError::IncorrectPayment {
            got: payment.u128(),
            expected: amount.u128(),
        });
    }

    Ok(Some(coin(amount.u128(), NATIVE_DENOM)))
}

pub fn invalid_char(c: char) -> bool {
    let is_valid = c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-';
    !is_valid
}
