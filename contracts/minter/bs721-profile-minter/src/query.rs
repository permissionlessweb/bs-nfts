use bs_profile::minter::{Config, SudoParams};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdResult};

use crate::{
    msg::QueryMsg,
    state::{ADMIN, CONFIG, PROFILE_COLLECTION, SUDO_PARAMS, WHITELISTS},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin {} => to_json_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::Collection {} => to_json_binary(&query_collection(deps)?),
        QueryMsg::Params {} => to_json_binary(&query_params(deps)?),
        QueryMsg::Whitelists {} => to_json_binary(&query_whitelists(deps)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

fn query_whitelists(deps: Deps) -> StdResult<Vec<Addr>> {
    let whitelists = WHITELISTS.load(deps.storage)?;
    Ok(whitelists.iter().map(|w| w.addr.clone()).collect())
}

fn query_collection(deps: Deps) -> StdResult<Addr> {
    PROFILE_COLLECTION.load(deps.storage)
}

fn query_params(deps: Deps) -> StdResult<SudoParams> {
    SUDO_PARAMS.load(deps.storage)
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}
