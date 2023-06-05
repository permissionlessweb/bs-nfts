#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
    Uint128, Uint64,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;

use crate::msg::ExecuteMsg;
use crate::state::DENOM;
use crate::{
    msg::{ContributorListResponse, ContributorResponse, InstantiateMsg, QueryMsg},
    state::{Contributor, CONTRIBUTORS, TOTAL_SHARES},
    ContractError,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:bs721-royalty";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    mut msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    msg.validate()?;

    let contributors = msg.contributors.clone();
    let mut total_shares = Uint64::zero();

    for contributor in contributors.into_iter() {
        let contributor_addr = deps.api.addr_validate(&contributor.address)?;
        let contributor_share = Uint64::from(contributor.share);
        total_shares = total_shares.checked_add(contributor_share).unwrap();

        CONTRIBUTORS.save(
            deps.storage,
            &contributor_addr,
            &Contributor {
                role: contributor.role,
                share: contributor.share,
            },
        )?;
    }

    TOTAL_SHARES.save(deps.storage, &total_shares.u64())?;
    DENOM.save(deps.storage, &msg.denom)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::WithdrawForAll {} => execute_withdraw_for_all(deps.as_ref(), env, info),
    }
}

pub fn execute_withdraw_for_all(
    deps: Deps,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // check if the sender is a contributor
    let sender_addr = info.sender.clone();
    let _sender = CONTRIBUTORS.load(deps.storage, &sender_addr);
    _sender.map_err(|_| ContractError::Unauthorized {})?;

    // get denom from storage
    let denom = DENOM.load(deps.storage)?;

    // get contract funds
    let funds = deps
        .querier
        .query_balance(env.contract.address, denom.clone())?;

    // check if contract funds is zero
    if funds.amount.is_zero() {
        return Err(ContractError::NoFunds {});
    }

    // get total shares
    let total_shares = TOTAL_SHARES.load(deps.storage)?;

    // calculate multiplier
    let multiplier = funds.amount / Uint128::from(total_shares);
    if multiplier.is_zero() {
        return Err(ContractError::NoFunds {});
    }

    // compute bank messages for all contributors
    let bank_msgs = CONTRIBUTORS
        .range(deps.storage, None, None, Order::Ascending)
        .into_iter()
        .filter(|item| item.as_ref().map_or(false, |(_, data)| data.share > 0))
        .map(|item| {
            let (addr, data) = item.unwrap();
            let amount = Uint128::from(data.share) * multiplier;
            BankMsg::Send {
                to_address: addr.into(),
                amount: vec![Coin {
                    denom: denom.clone(),
                    amount,
                }],
            }
        })
        .collect::<Vec<_>>();

    Ok(Response::new()
        .add_attribute("action", "withdraw_for_all")
        .add_messages(bank_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ListContributors { start_after, limit } => {
            to_binary(&query_list_contributors(deps, start_after, limit)?)
        }
    }
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn query_list_contributors(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ContributorListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let addr = maybe_addr(deps.api, start_after)?;
    let start = addr.as_ref().map(Bound::exclusive);

    let items = CONTRIBUTORS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(addr, data)| ContributorResponse {
                address: addr.into(),
                role: data.role,
                share: data.share,
            })
        })
        .collect::<StdResult<_>>()?;

    Ok(ContributorListResponse {
        contributors: items,
    })
}
