use bs721_accounts::ExecuteMsg;
use bs_account::{
    common::charge_fees, market::ExecuteMsg as MarketplaceExecuteMsg, minter::Config, Metadata,
};
use bs_std::NATIVE_DENOM;
use cosmwasm_std::to_json_binary;
use cosmwasm_std::{
    coin, Coin, Decimal, DepsMut, Env, Event, MessageInfo, Response, Uint128, WasmMsg,
};
use cw_utils::must_pay;

use crate::state::ACCOUNT_MARKETPLACE;
use crate::{
    state::{ACCOUNT_COLLECTION, ADMIN, CONFIG, PAUSED, SUDO_PARAMS},
    ContractError,
};

pub fn execute_mint_and_list(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    account: &str,
) -> Result<Response, ContractError> {
    if PAUSED.load(deps.storage)? {
        return Err(ContractError::MintingPaused {});
    }

    let sender = &info.sender.to_string();
    let config = CONFIG.load(deps.storage)?;
    let params = SUDO_PARAMS.load(deps.storage)?;

    if env.block.time < config.public_mint_start_time {
        return Err(ContractError::MintingNotStarted {});
    }

    validate_name(account, params.min_name_length, params.max_name_length)?;
    let price = validate_payment(account.len(), &info, params.base_price.u128())?;

    let mut res = Response::new();
    // burns any tokens sent as fees if required
    if price.is_some() {
        charge_fees(&mut res, price.clone().unwrap().amount);
    }

    let marketplace = ACCOUNT_MARKETPLACE.load(deps.storage)?;
    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;

    // mint token
    let mint_msg = ExecuteMsg::Mint {
        token_id: account.to_string(),
        owner: sender.to_string(),
        token_uri: None,
        extension: Metadata::default(),
        seller_fee_bps: None,
        payment_addr: None,
    };
    let mint_msg_exec = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_json_binary(&mint_msg)?,
        funds: vec![],
    };

    let ask_msg = MarketplaceExecuteMsg::SetAsk {
        token_id: account.to_string(),
        seller: sender.to_string(),
    };
    let list_msg_exec = WasmMsg::Execute {
        contract_addr: marketplace.to_string(),
        msg: to_json_binary(&ask_msg)?,
        funds: vec![],
    };

    let event = Event::new("mint-and-list")
        .add_attribute("name", account)
        .add_attribute("owner", sender)
        .add_attribute(
            "price",
            price
                .unwrap_or_else(|| coin(0u128, NATIVE_DENOM))
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

    if len > 4u32 && name[2..4].contains("--") {
        return Err(ContractError::InvalidName {});
    }

    Ok(())
}

pub enum Discount {
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
