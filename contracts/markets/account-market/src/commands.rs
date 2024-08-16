use crate::helpers::{get_renewal_price_and_bid, renew_name};
use crate::{
    error::ContractError,
    hooks::{prepare_ask_hook, prepare_bid_hook, prepare_sale_hook},
};
use btsg_account::{
    common::{charge_fees, SECONDS_PER_YEAR},
    minter::SudoParams as BsProfileMinterSudoParams,
};
use btsg_account::{
    market::{state::*, *},
    minter::BsAccountMinterQueryMsg,
};
use bs_std::NATIVE_DENOM;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, coins, ensure, to_json_binary, Addr, BankMsg, Coin, Decimal, Deps, DepsMut, Empty, Env,
    Event, MessageInfo, Order, Response, StdError, StdResult, Storage, SubMsg, SubMsgResult,
    Timestamp, Uint128, WasmMsg,
};
use std::marker::PhantomData;

use cw721::{Cw721ExecuteMsg, OwnerOfResponse};
use cw721_base::helpers::Cw721Contract;
use cw_storage_plus::Bound;
use cw_utils::{may_pay, must_pay, nonpayable};

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 100;
pub const PROPOSE_BIDDER_A: u64 = 1;
pub const ACCEPT_BIDDER_A: u64 = 2;
pub const PROPOSE_BIDDER_B: u64 = 3;
pub const ACCEPT_BIDDER_B: u64 = 4;

/// Setup this contract (can be run once only)
pub fn execute_setup(
    deps: DepsMut,
    minter: Addr,
    collection: Addr,
) -> Result<Response, ContractError> {
    if IS_SETUP.load(deps.storage)? {
        return Err(ContractError::AlreadySetup {});
    }
    ACCOUNT_MINTER.save(deps.storage, &minter)?;
    ACCOUNT_COLLECTION.save(deps.storage, &collection)?;
    IS_SETUP.save(deps.storage, &true)?;

    let event = Event::new("setup")
        .add_attribute("minter", minter)
        .add_attribute("collection", collection);
    Ok(Response::new().add_event(event))
}

/// A seller may set an Ask on their NFT to list it on Marketplace
pub fn execute_set_ask(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: &str,
    seller: Addr,
) -> Result<Response, ContractError> {
    let minter = ACCOUNT_MINTER.load(deps.storage)?;
    if info.sender != minter {
        return Err(ContractError::UnauthorizedMinter {});
    }

    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;

    // check if collection is approved to transfer on behalf of the seller (needs is_minter_admin == true)
    let ops = Cw721Contract::<Empty, Empty>(collection, PhantomData, PhantomData).all_operators(
        &deps.querier,
        seller.to_string(),
        false,
        None,
        None,
    )?;
    if ops.is_empty() {
        return Err(ContractError::NotApproved {});
    }

    let renewal_time = env.block.time.plus_seconds(31536000u64);

    let ask = Ask {
        token_id: token_id.to_string(),
        id: increment_asks(deps.storage)?,
        seller: seller.clone(),
        renewal_time,
        renewal_fund: Uint128::zero(),
    };
    store_ask(deps.storage, &ask)?;

    RENEWAL_QUEUE.save(
        deps.storage,
        (renewal_time.seconds(), ask.id),
        &token_id.to_string(),
    )?;

    let hook = prepare_ask_hook(deps.as_ref(), &ask, HookAction::Create)?;

    let event = Event::new("set-ask")
        .add_attribute("token_id", token_id)
        .add_attribute("ask_id", ask.id.to_string())
        .add_attribute("renewal_time", renewal_time.to_string())
        .add_attribute("seller", seller);

    Ok(Response::new().add_event(event).add_submessages(hook))
}

/// Removes the ask on a particular NFT
pub fn execute_remove_ask(
    deps: DepsMut,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    // `ask` can only be removed by burning from the collection
    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;
    if info.sender != collection {
        return Err(ContractError::Unauthorized {});
    }

    // don't allow burning if ask has bids on it
    let bid_count = bids()
        .prefix(token_id.to_string())
        .keys(deps.storage, None, None, Order::Ascending)
        .count();
    if bid_count > 0 {
        return Err(ContractError::ExistingBids {});
    }

    let key = ask_key(token_id);
    let ask = asks().load(deps.storage, key.clone())?;
    asks().remove(deps.storage, key)?;

    RENEWAL_QUEUE.remove(deps.storage, (ask.renewal_time.seconds(), ask.id));

    let hook = prepare_ask_hook(deps.as_ref(), &ask, HookAction::Delete)?;

    let event = Event::new("remove-ask").add_attribute("token_id", token_id);

    Ok(Response::new().add_event(event).add_submessages(hook))
}

/// When an NFT is transferred, the `ask` has to be updated with the new
/// seller. Also any renewal funds should be refunded to the previous owner.
pub fn execute_update_ask(
    deps: DepsMut,
    info: MessageInfo,
    token_id: &str,
    seller: Addr,
) -> Result<Response, ContractError> {
    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;
    if info.sender != collection {
        return Err(ContractError::Unauthorized {});
    }

    let mut res = Response::new();

    // refund any renewal funds and update the seller
    let mut ask = asks().load(deps.storage, ask_key(token_id))?;
    if !ask.renewal_fund.is_zero() {
        let msg = BankMsg::Send {
            to_address: ask.seller.to_string(),
            amount: coins(ask.renewal_fund.u128(), NATIVE_DENOM),
        };
        res = res.add_message(msg);
        ask.renewal_fund = Uint128::zero();
    }
    ask.seller = seller.clone();
    asks().save(deps.storage, ask_key(token_id), &ask)?;

    let event = Event::new("update-ask")
        .add_attribute("token_id", token_id)
        .add_attribute("seller", seller);

    Ok(res.add_event(event))
}

/// Places a bid on a name. The bid is escrowed in the contract.
pub fn execute_set_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    let params = SUDO_PARAMS.load(deps.storage)?;

    let ask_key = ask_key(token_id);
    asks().load(deps.storage, ask_key)?;

    let bid_price = must_pay(&info, NATIVE_DENOM)?;
    if bid_price < params.min_price {
        return Err(ContractError::PriceTooSmall(bid_price));
    }

    let bidder = info.sender;
    let mut res = Response::new();
    let bid_key = bid_key(token_id, &bidder);

    if let Some(existing_bid) = bids().may_load(deps.storage, bid_key.clone())? {
        bids().remove(deps.storage, bid_key)?;
        let refund_bidder = BankMsg::Send {
            to_address: bidder.to_string(),
            amount: vec![coin(existing_bid.amount.u128(), NATIVE_DENOM)],
        };
        res = res.add_message(refund_bidder)
    }

    let bid = Bid::new(token_id, bidder.clone(), bid_price, env.block.time);
    store_bid(deps.storage, &bid)?;

    let hook = prepare_bid_hook(deps.as_ref(), &bid.clone(), HookAction::Create)?;

    let event = Event::new("set-bid")
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bidder)
        .add_attribute("bid_price", bid_price.to_string());

    Ok(res
        .add_event(event)
        // .add_message(execute)
        .add_submessages(hook))
}

/// Removes a bid made by the bidder. Bidders can only remove their own bids
pub fn execute_remove_bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let bidder = info.sender;

    let key = bid_key(token_id, &bidder);
    let bid = bids().load(deps.storage, key.clone())?;
    bids().remove(deps.storage, key)?;

    let refund_bidder_msg = BankMsg::Send {
        to_address: bid.bidder.to_string(),
        amount: vec![coin(bid.amount.u128(), NATIVE_DENOM)],
    };

    let hook = prepare_bid_hook(deps.as_ref(), &bid, HookAction::Delete)?;

    let event = Event::new("remove-bid")
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bidder);

    let res = Response::new()
        .add_message(refund_bidder_msg)
        .add_submessages(hook)
        .add_event(event);

    Ok(res)
}

/// Seller can accept a bid which transfers funds as well as the token.
/// The bid is removed, then a new ask is created for the same token.
pub fn execute_accept_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: &str,
    bidder: Addr,
) -> Result<Response, ContractError> {
    println!("1.0 execute_accept bid ----------------------------");
    nonpayable(&info)?;
    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;
    only_owner(deps.as_ref(), &info, &collection, token_id)?;

    let ask_key = ask_key(token_id);
    let bid_key = bid_key(token_id, &bidder);

    let ask = asks().load(deps.storage, ask_key)?;
    let bid = bids().load(deps.storage, bid_key.clone())?;

    // Check if token is approved for transfer
    Cw721Contract::<Empty, Empty>(collection, PhantomData, PhantomData).approval(
        &deps.querier,
        token_id,
        info.sender.as_ref(),
        None,
    )?;

    // Remove accepted bid
    bids().remove(deps.storage, bid_key)?;

    // Update renewal queue
    let renewal_time = env.block.time.plus_seconds(SECONDS_PER_YEAR);
    RENEWAL_QUEUE.save(
        deps.storage,
        (renewal_time.seconds(), ask.id),
        &token_id.to_string(),
    )?;

    let mut res = Response::new();

    // Return renewal funds if there's any
    if !ask.renewal_fund.is_zero() {
        let msg = BankMsg::Send {
            to_address: ask.seller.to_string(),
            amount: coins(ask.renewal_fund.u128(), NATIVE_DENOM),
        };
        res = res.add_message(msg);
    }

    // Transfer funds and NFT
    finalize_sale(
        deps.as_ref(),
        ask.clone(),
        bid.amount,
        bidder.clone(),
        &mut res,
    )?;

    // Update Ask with new seller and renewal time
    let ask = Ask {
        token_id: token_id.to_string(),
        id: ask.id,
        seller: bidder.clone(),
        renewal_time,
        renewal_fund: Uint128::zero(),
    };
    store_ask(deps.storage, &ask)?;

    let event = Event::new("accept-bid")
        .add_attribute("token_id", token_id)
        .add_attribute("bidder", bidder)
        .add_attribute("price", bid.amount.to_string());

    Ok(res.add_event(event))
}

pub fn execute_fund_renewal(
    deps: DepsMut,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    let payment = must_pay(&info, NATIVE_DENOM)?;

    let mut ask = asks().load(deps.storage, ask_key(token_id))?;
    ask.renewal_fund += payment;
    asks().save(deps.storage, ask_key(token_id), &ask)?;

    let event = Event::new("fund-renewal")
        .add_attribute("token_id", token_id)
        .add_attribute("payment", payment);
    Ok(Response::new().add_event(event))
}

pub fn execute_refund_renewal(
    deps: DepsMut,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut ask = asks().load(deps.storage, ask_key(token_id))?;

    if ask.seller != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    if ask.renewal_fund.is_zero() {
        return Err(ContractError::NoRenewalFund {});
    }

    let msg = BankMsg::Send {
        to_address: ask.seller.to_string(),
        amount: vec![coin(ask.renewal_fund.u128(), NATIVE_DENOM)],
    };

    ask.renewal_fund = Uint128::zero();
    asks().save(deps.storage, ask_key(token_id), &ask)?;

    let event = Event::new("refund-renewal")
        .add_attribute("token_id", token_id)
        .add_attribute("refund", ask.renewal_fund);
    Ok(Response::new().add_event(event).add_message(msg))
}

pub fn execute_renew(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: &str,
) -> Result<Response, ContractError> {
    let mut ask = asks().load(deps.storage, ask_key(token_id))?;
    let sudo_params = SUDO_PARAMS.load(deps.storage)?;

    let ask_renew_start_time = ask.renewal_time.seconds() - sudo_params.renew_window;

    ensure!(
        env.block.time.seconds() >= ask_renew_start_time,
        ContractError::CannotProcessFutureRenewal {}
    );

    let name_minter = ACCOUNT_MINTER.load(deps.storage)?;
    let name_minter_params = deps.querier.query_wasm_smart::<BsProfileMinterSudoParams>(
        name_minter,
        &BsAccountMinterQueryMsg::Params {},
    )?;

    let (renewal_price, _valid_bid) = get_renewal_price_and_bid(
        deps.as_ref(),
        &env.block.time,
        &sudo_params,
        &ask.token_id,
        name_minter_params.base_price.u128(),
    )?;

    let payment = may_pay(&info, NATIVE_DENOM)?;

    ask.renewal_fund += payment;

    ensure!(
        ask.renewal_fund >= renewal_price,
        ContractError::InsufficientRenewalFunds {
            expected: coin(renewal_price.u128(), NATIVE_DENOM),
            actual: coin(ask.renewal_fund.u128(), NATIVE_DENOM),
        }
    );

    let mut response = Response::new();

    response = renew_name(deps, &env, &sudo_params, ask, renewal_price, response)?;

    Ok(response)
}

/// Anyone can call this to process renewals for a block and earn a reward
pub fn execute_process_renewal(
    _deps: DepsMut,
    env: Env,
    time: Timestamp,
) -> Result<Response, ContractError> {
    println!("Processing renewals at time {}", time);

    if time > env.block.time {
        return Err(ContractError::CannotProcessFutureRenewal {});
    }

    // // TODO: add renewal processing logic
    // let renewal_queue = RENEWAL_QUEUE.load(deps.storage, time)?;
    // for name in renewal_queue.iter() {
    //     let ask = asks().load(deps.storage, ask_key(name))?;
    //     if ask.renewal_fund.is_zero() {
    //         continue;
    //         // transfer ownership to name service
    //         // list in marketplace for 0.5% of bid price
    //         // if no bids, list for original price
    //     }

    //     // charge renewal fee
    //     // pay out reward to operator
    //     // reset ask

    //     // Update Ask with new renewal_time
    //     let renewal_time = env.block.time.plus_seconds(SECONDS_PER_YEAR);
    //     let ask = Ask {
    //         token_id: name.to_string(),
    //         id: ask.id,
    //         seller: ask.seller,
    //         renewal_time,
    //         renewal_fund: ask.renewal_fund - payment, // validate payment
    //     };
    //     store_ask(deps.storage, &ask)?;
    // }

    let event = Event::new("process-renewal").add_attribute("time", time.to_string());
    Ok(Response::new().add_event(event))
}

/// Transfers funds and NFT, updates bid
fn finalize_sale(
    deps: Deps,
    ask: Ask,
    price: Uint128,
    buyer: Addr,
    res: &mut Response,
) -> StdResult<()> {
    println!("1.1 finalize sale ----------------------------");
    payout(deps, price, ask.seller.clone(), res)?;

    let cw721_transfer_msg = Cw721ExecuteMsg::TransferNft {
        token_id: ask.token_id.to_string(),
        recipient: buyer.to_string(),
    };

    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;

    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_json_binary(&cw721_transfer_msg)?,
        funds: vec![],
    };
    res.messages.push(SubMsg::new(exec_cw721_transfer));

    res.messages
        .append(&mut prepare_sale_hook(deps, &ask, buyer.clone())?);

    let event = Event::new("finalize-sale")
        .add_attribute("token_id", ask.token_id.to_string())
        .add_attribute("seller", ask.seller.to_string())
        .add_attribute("buyer", buyer.to_string())
        .add_attribute("price", price.to_string());
    res.events.push(event);

    Ok(())
}

/// Payout a bid
fn payout(
    deps: Deps,
    payment: Uint128,
    payment_recipient: Addr,
    res: &mut Response,
) -> StdResult<()> {
    let params = SUDO_PARAMS.load(deps.storage)?;

    let fee = payment * params.trading_fee_percent;
    if fee > payment {
        return Err(StdError::generic_err("Fees exceed payment"));
    }
    charge_fees(res, fee);

    // pay seller
    let seller_share_msg = BankMsg::Send {
        to_address: payment_recipient.to_string(),
        amount: vec![coin((payment - fee).u128(), NATIVE_DENOM.to_string())],
    };
    res.messages.push(SubMsg::new(seller_share_msg));

    Ok(())
}

fn store_bid(store: &mut dyn Storage, bid: &Bid) -> StdResult<()> {
    bids().save(store, bid_key(&bid.token_id, &bid.bidder), bid)
}

pub fn store_ask(store: &mut dyn Storage, ask: &Ask) -> StdResult<()> {
    asks().save(store, ask_key(&ask.token_id), ask)
}

/// Checks to enfore only NFT owner can call
fn only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
) -> Result<OwnerOfResponse, ContractError> {
    let res = Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData)
        .owner_of(&deps.querier, token_id, false)?;
    if res.owner != info.sender {
        return Err(ContractError::UnauthorizedOwner {});
    }

    Ok(res)
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let minter = ACCOUNT_MINTER.load(deps.storage)?;
    let collection = ACCOUNT_COLLECTION.load(deps.storage)?;

    Ok(ConfigResponse { minter, collection })
}

pub fn query_renewal_queue(deps: Deps, time: Timestamp) -> StdResult<Vec<Ask>> {
    let names = RENEWAL_QUEUE
        .prefix(time.seconds())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    names
        .iter()
        .map(|name| asks().load(deps.storage, ask_key(name)))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_asks(
    deps: Deps,
    start_after: Option<btsg_account::market::state::Id>,
    limit: Option<u32>,
) -> StdResult<Vec<Ask>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    asks()
        .idx
        .id
        .range(
            deps.storage,
            Some(Bound::exclusive(start_after.unwrap_or_default())),
            None,
            Order::Ascending,
        )
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_ask_count(deps: Deps) -> StdResult<u64> {
    ASK_COUNT.load(deps.storage)
}

// TODO: figure out how to paginate by `Id` instead of `TokenId`
pub fn query_asks_by_seller(
    deps: Deps,
    seller: Addr,
    start_after: Option<TokenId>,
    limit: Option<u32>,
) -> StdResult<Vec<Ask>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = start_after.map(|start| Bound::exclusive(ask_key(&start)));

    asks()
        .idx
        .seller
        .prefix(seller)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_ask_renew_prices(
    deps: Deps,
    current_time: Timestamp,
    token_ids: Vec<String>,
) -> StdResult<Vec<AskRenewPriceResponse>> {
    token_ids
        .iter()
        .map(|token_id| {
            let (coin_option, bid_option) =
                query_ask_renew_price(deps, current_time, token_id.to_string())?;
            Ok(AskRenewPriceResponse {
                token_id: token_id.to_string(),
                price: coin_option.unwrap_or_default(),
                bid: bid_option,
            })
        })
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_ask(deps: Deps, token_id: TokenId) -> StdResult<Option<Ask>> {
    asks().may_load(deps.storage, ask_key(&token_id))
}

pub fn query_bid(deps: Deps, token_id: TokenId, bidder: Addr) -> StdResult<Option<Bid>> {
    bids().may_load(deps.storage, (token_id, bidder))
}

pub fn query_bids_by_bidder(
    deps: Deps,
    bidder: Addr,
    start_after: Option<TokenId>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start = start_after.map(|start| Bound::exclusive((start, bidder.clone())));

    bids()
        .idx
        .bidder
        .prefix(bidder)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_bids_for_seller(
    deps: Deps,
    seller: Addr,
    start_after: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    // Query seller asks, then collect bids starting after token_id
    // Limitation: Can not collect bids in the middle using `start_after: token_id` pattern
    // This leads to imprecise pagination based on token id and not bid count
    let start_token_id =
        start_after.map(|start| Bound::<AskKey>::exclusive(ask_key(&start.token_id)));

    let bids = asks()
        .idx
        .seller
        .prefix(seller)
        .range(deps.storage, start_token_id, None, Order::Ascending)
        .take(limit)
        .map(|res| res.map(|item| item.0).unwrap())
        .flat_map(|token_id| {
            bids()
                .prefix(token_id)
                .range(deps.storage, None, None, Order::Ascending)
                .flat_map(|item| item.map(|(_, b)| b))
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(bids)
}

pub fn query_bids(
    deps: Deps,
    token_id: TokenId,
    start_after: Option<Bidder>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    bids()
        .prefix(token_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_highest_bid(deps: Deps, token_id: TokenId) -> StdResult<Option<Bid>> {
    let bid = bids()
        .idx
        .price
        .range(deps.storage, None, None, Order::Descending)
        .filter_map(|item| {
            let (key, bid) = item.unwrap();
            if key.0 == token_id {
                Some(bid)
            } else {
                None
            }
        })
        .take(1)
        .collect::<Vec<_>>()
        .first()
        .cloned();

    Ok(bid)
}

pub fn query_bids_sorted_by_price(
    deps: Deps,
    start_after: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let start: Option<Bound<(u128, BidKey)>> = start_after.map(|offset| {
        Bound::exclusive((
            offset.price.u128(),
            bid_key(&offset.token_id, &offset.bidder),
        ))
    });

    bids()
        .idx
        .price
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn reverse_query_bids_sorted_by_price(
    deps: Deps,
    start_before: Option<BidOffset>,
    limit: Option<u32>,
) -> StdResult<Vec<Bid>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let end: Option<Bound<(u128, BidKey)>> = start_before.map(|offset| {
        Bound::exclusive((
            offset.price.u128(),
            bid_key(&offset.token_id, &offset.bidder),
        ))
    });

    bids()
        .idx
        .price
        .range(deps.storage, None, end, Order::Descending)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()
}

pub fn query_params(deps: Deps) -> StdResult<SudoParams> {
    SUDO_PARAMS.load(deps.storage)
}

// pub fn query_asks_by_renew_time(
//     deps: Deps,
//     max_time: Timestamp,
//     start_after: Option<Timestamp>,
//     limit: Option<u32>,
// ) -> StdResult<Vec<Ask>> {
//     let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT) as usize;

//     let renewable_asks = asks()
//         .idx
//         .renewal_time
//         .range(
//             deps.storage,
//             start_after.map(|start| Bound::inclusive((start.seconds() + 1, "".to_string()))),
//             Some(Bound::exclusive((max_time.seconds() + 1, "".to_string()))),
//             Order::Ascending,
//         )
//         .take(limit)
//         .map(|item| item.map(|(_, v)| v))
//         .collect::<StdResult<Vec<_>>>()?;

//     Ok(renewable_asks)
// }

pub fn query_ask_renew_price(
    deps: Deps,
    current_time: Timestamp,
    token_id: String,
) -> StdResult<(Option<Coin>, Option<Bid>)> {
    let ask = asks().load(deps.storage, ask_key(&token_id))?;
    let sudo_params = SUDO_PARAMS.load(deps.storage)?;

    let ask_renew_start_time = ask.renewal_time.seconds() - sudo_params.renew_window;

    if current_time.seconds() < ask_renew_start_time {
        return Ok((None, None));
    }

    let name_minter = ACCOUNT_MINTER.load(deps.storage)?;
    let name_minter_params = deps.querier.query_wasm_smart::<BsProfileMinterSudoParams>(
        name_minter,
        &(BsAccountMinterQueryMsg::Params {}),
    )?;

    let (renewal_price, valid_bid) = get_renewal_price_and_bid(
        deps,
        &current_time,
        &sudo_params,
        &ask.token_id,
        name_minter_params.base_price.u128(),
    )
    .map_err(|_| StdError::generic_err("failed to fetch renewal price".to_string()))?;

    Ok((Some(coin(renewal_price.u128(), NATIVE_DENOM)), valid_bid))
}

/// Only governance can update contract params
pub fn sudo_update_params(
    deps: DepsMut,
    _env: Env,
    param_info: ParamInfo,
) -> Result<Response, ContractError> {
    let ParamInfo {
        trading_fee_bps,
        min_price,
        ask_interval,
    } = param_info;
    if let Some(trading_fee_bps) = trading_fee_bps {
        if trading_fee_bps > MAX_FEE_BPS {
            return Err(ContractError::InvalidTradingFeeBps(trading_fee_bps));
        }
    }

    let mut params = SUDO_PARAMS.load(deps.storage)?;

    params.trading_fee_percent = trading_fee_bps
        .map(|bps| Decimal::percent(bps) / Uint128::from(100u128))
        .unwrap_or(params.trading_fee_percent);

    params.min_price = min_price.unwrap_or(params.min_price);

    params.ask_interval = ask_interval.unwrap_or(params.ask_interval);

    SUDO_PARAMS.save(deps.storage, &params)?;

    let event = Event::new("update-params")
        .add_attribute(
            "trading_fee_percent",
            params.trading_fee_percent.to_string(),
        )
        .add_attribute("min_price", params.min_price);
    Ok(Response::new().add_event(event))
}

pub fn sudo_update_name_minter(deps: DepsMut, collection: Addr) -> Result<Response, ContractError> {
    ACCOUNT_MINTER.save(deps.storage, &collection)?;

    let event = Event::new("update-account-minter").add_attribute("minter", collection);
    Ok(Response::new().add_event(event))
}

pub fn sudo_update_name_collection(
    deps: DepsMut,
    collection: Addr,
) -> Result<Response, ContractError> {
    ACCOUNT_COLLECTION.save(deps.storage, &collection)?;

    let event = Event::new("update-account-collection").add_attribute("collection", collection);
    Ok(Response::new().add_event(event))
}

pub fn sudo_add_sale_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    SALE_HOOKS.add_hook(deps.storage, hook.clone())?;

    let event = Event::new("add-sale-hook").add_attribute("hook", hook);
    Ok(Response::new().add_event(event))
}

pub fn sudo_add_ask_hook(deps: DepsMut, _env: Env, hook: Addr) -> Result<Response, ContractError> {
    ASK_HOOKS.add_hook(deps.storage, hook.clone())?;

    let event = Event::new("add-ask-hook").add_attribute("hook", hook);
    Ok(Response::new().add_event(event))
}

pub fn sudo_add_bid_hook(deps: DepsMut, _env: Env, hook: Addr) -> Result<Response, ContractError> {
    BID_HOOKS.add_hook(deps.storage, hook.clone())?;

    let event = Event::new("add-bid-hook").add_attribute("hook", hook);
    Ok(Response::new().add_event(event))
}

pub fn sudo_remove_sale_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    SALE_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let event = Event::new("remove-sale-hook").add_attribute("hook", hook);
    Ok(Response::new().add_event(event))
}

pub fn sudo_remove_ask_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    ASK_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let event = Event::new("remove-ask-hook").add_attribute("hook", hook);
    Ok(Response::new().add_event(event))
}

pub fn sudo_remove_bid_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    BID_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let event = Event::new("remove-bid-hook").add_attribute("hook", hook);
    Ok(Response::new().add_event(event))
}

/// Propose the marketplace as owner for escrow of account
fn _propose_accepted_bidder_a(_deps: Deps, _env: Env, _res: &mut Response) -> StdResult<()> {
    // propose owner as marketplace for escrow purposes
    Ok(())
}

pub(crate) fn propose_accepted_bidder_a_response(
    _env: Env,
    _deps: DepsMut,
    _result: SubMsgResult,
) -> Result<Response, ContractError> {
    println!("1.5 propose_accepted_bidder_a ----------------------------");

    let res = Response::new();

    Ok(res)
}
