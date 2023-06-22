#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, BalanceResponse, BankMsg, Binary, Coin, Decimal, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;

use crate::msg::ExecuteMsg;
use crate::state::{DENOM, WITHDRAWABLE_AMOUNT};
use crate::{
    msg::{ContributorListResponse, ContributorResponse, InstantiateMsg, QueryMsg},
    state::{Contributor, CONTRIBUTORS, TOTAL_SHARES},
    ContractError,
};

// version and name info for migration
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

    let total_shares = msg.validate_and_compute_total_shares()?;

    for contributor in msg.contributors {
        let contributor_addr = deps.api.addr_validate(&contributor.address)?;
        let percentage_shares =
            Decimal::from_ratio(Uint128::from(contributor.shares), total_shares);

        CONTRIBUTORS.save(
            deps.storage,
            &contributor_addr,
            &Contributor {
                role: contributor.role,
                initial_shares: contributor.shares,
                percentage_shares,
                withdrawable_amount: Uint128::zero(),
            },
        )?;
    }

    WITHDRAWABLE_AMOUNT.save(deps.storage, &Uint128::zero())?;
    TOTAL_SHARES.save(deps.storage, &total_shares)?;
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
        ExecuteMsg::Distribute {} => execute_distribute(deps, env),
        ExecuteMsg::Withdraw {} => execute_withdraw(deps, info),
        ExecuteMsg::WithdrawForAll {} => unimplemented!(), //execute_withdraw_for_all(deps.as_ref(), env, info),
    }
}

pub fn execute_distribute(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    // get contract funds
    let funds = deps
        .querier
        .query_balance(env.contract.address, DENOM.load(deps.storage)?)?;

    let withdrawable_amount = WITHDRAWABLE_AMOUNT.load(deps.storage).unwrap_or_default();

    let distributable_royalties = funds.amount.saturating_sub(withdrawable_amount);
    if distributable_royalties.is_zero() {
        return Err(ContractError::NothingToDistribute {});
    }

    let multiplier = compute_shares_multiplier(deps.as_ref(), funds.amount)?;
    // we have to handle situation with very few denom to distribute
    if multiplier.is_zero() {
        return Err(ContractError::NothingToDistribute {});
    }

    let contributors = CONTRIBUTORS
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<Addr>>>()?;
    let mut distributed_shares = Uint128::zero();
    for contributor_address in contributors {
        CONTRIBUTORS.update(
            deps.storage,
            &contributor_address,
            |info| -> Result<_, ContractError> {
                let mut info = info.unwrap();
                let contributor_shares = funds.amount * multiplier;
                info.withdrawable_amount = info
                    .withdrawable_amount
                    .checked_add(contributor_shares)
                    .map_err(ContractError::OverflowErr)?;
                distributed_shares = distributed_shares.checked_add(contributor_shares)?;
                Ok(info)
            },
        )?;
    }

    WITHDRAWABLE_AMOUNT.update(deps.storage, |amount| -> Result<Uint128, ContractError> {
        amount
            .checked_add(distributed_shares)
            .map_err(ContractError::OverflowErr)
    })?;

    Ok(Response::new().add_attribute("action", "execute_distribute"))
}

pub fn execute_withdraw(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    // only contributor can withdraw
    let maybe_contributor = CONTRIBUTORS.load(deps.storage, &info.sender);
    if maybe_contributor.is_err() {
        return Err(ContractError::Unauthorized {});
    }

    let mut tokens_to_send: Coin = Coin::new(0u128, DENOM.load(deps.storage)?);
    CONTRIBUTORS.update(
        deps.storage,
        &info.sender,
        |c| -> Result<_, ContractError> {
            let mut contributor = c.unwrap();
            tokens_to_send.amount = tokens_to_send
                .amount
                .checked_add(contributor.withdrawable_amount)?;
            contributor.withdrawable_amount = Uint128::zero();
            Ok(contributor)
        },
    )?;

    WITHDRAWABLE_AMOUNT.update(deps.storage, |amount| -> Result<_, ContractError> {
        Ok(amount.checked_sub(tokens_to_send.amount)?)
    })?;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![tokens_to_send.clone()],
    };

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "withdraw"),
            ("contributor", info.sender.as_ref()),
            ("amount", &format!("{}", tokens_to_send.amount)),
        ])
        .add_message(msg))
}

pub fn compute_shares_multiplier(deps: Deps, shares: Uint128) -> Result<Uint128, ContractError> {
    // get total shares
    let total_shares = TOTAL_SHARES.load(deps.storage)?;

    // calculate multiplier
    let multiplier = shares / Uint128::from(total_shares);
    Ok(multiplier)
}

/* /// Diatribute contract denom balance to all contributors according to their shares.
pub fn execute_withdraw_for_all(
    deps: Deps,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // check if the sender is a contributor
    let sender_addr = info.sender;
    let _sender = CONTRIBUTORS.load(deps.storage, &sender_addr);
    _sender.map_err(|_| ContractError::Unauthorized {})?;

    // get contract funds
    let funds = deps
        .querier
        .query_balance(env.contract.address, DENOM.load(deps.storage)?)?;

    let multiplier = compute_shares_multiplier(deps, funds.amount)?;

    // compute bank messages for all contributors
    let bank_msgs = CONTRIBUTORS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (addr, data) = item.unwrap();
            let amount = Uint128::from(data.share) * multiplier;
            BankMsg::Send {
                to_address: addr.into(),
                amount: vec![Coin {
                    denom: funds.denom.clone(),
                    amount,
                }],
            }
        })
        .collect::<Vec<_>>();

    Ok(Response::new()
        .add_attribute("action", "withdraw_for_all")
        .add_messages(bank_msgs))
} */

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ListContributors { start_after, limit } => {
            to_binary(&query_list_contributors(deps, start_after, limit)?)
        }
        QueryMsg::WithdrawableAmount {} => to_binary(&WITHDRAWABLE_AMOUNT.load(deps.storage)?),
        QueryMsg::DistributableAmount {} => to_binary(&query_distributable_amount(deps, env)?),
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
                initial_shares: data.initial_shares,
                percentage_shares: data.percentage_shares,
            })
        })
        .collect::<StdResult<_>>()?;

    Ok(ContributorListResponse {
        contributors: items,
    })
}

/// Returns the difference between contract balance and the amount of tokens that can be withdrawn as
/// royalties.
pub fn query_distributable_amount(deps: Deps, env: Env) -> StdResult<Uint128> {
    // get contract funds
    let funds = deps
        .querier
        .query_balance(env.contract.address, DENOM.load(deps.storage)?)?;

    let withdrawable_amount = WITHDRAWABLE_AMOUNT.load(deps.storage).unwrap_or_default();

    Ok(funds.amount.saturating_sub(withdrawable_amount))
}
