#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, DepsMut, Env, Event, Response, Uint128};
use bs_profile::minter::SudoParams;

use crate::{
    msg::SudoMsg,
    state::{NAME_COLLECTION, profile_marketplace, SUDO_PARAMS},
    ContractError,
};

pub fn sudo_update_params(
    deps: DepsMut,
    min_name_length: u32,
    max_name_length: u32,
    base_price: Uint128,
    fair_burn_bps: u64,
) -> Result<Response, ContractError> {
    SUDO_PARAMS.save(
        deps.storage,
        &SudoParams {
            min_name_length,
            max_name_length,
            base_price,
            // fair_burn_percent: Decimal::percent(fair_burn_bps) / Uint128::from(100u128),
        },
    )?;

    Ok(Response::new().add_attribute("action", "sudo_update_params"))
}

pub fn sudo_update_name_collection(
    deps: DepsMut,
    collection: Addr,
) -> Result<Response, ContractError> {
    NAME_COLLECTION.save(deps.storage, &collection)?;

    let event = Event::new("update-name-collection").add_attribute("collection", collection);
    Ok(Response::new().add_event(event))
}

pub fn sudo_update_profile_marketplace(
    deps: DepsMut,
    marketplace: Addr,
) -> Result<Response, ContractError> {
    profile_marketplace.save(deps.storage, &marketplace)?;

    let event = Event::new("update-name-marketplace").add_attribute("marketplace", marketplace);
    Ok(Response::new().add_event(event))
}