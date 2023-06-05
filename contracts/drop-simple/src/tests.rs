use bs721_royalty::msg::ContributorMsg;
use cosmwasm_std::{
    coin,
    testing::{mock_dependencies, mock_env, mock_info},
    DepsMut, Timestamp,
};
use cw_utils::Duration;

use crate::{
    contract::{instantiate, query_get_config},
    msg::{ExecuteMsg, InstantiateMsg},
    ContractError,
};

fn contributors() -> Vec<ContributorMsg> {
    vec![ContributorMsg {
        address: "contributor1".into(),
        role: "role".into(),
        share: 10,
    }]
}

fn duplicate_contributors() -> Vec<ContributorMsg> {
    vec![
        ContributorMsg {
            address: "contributor1".into(),
            role: "role".into(),
            share: 10,
        },
        ContributorMsg {
            address: "contributor1".into(),
            role: "role".into(),
            share: 10,
        },
    ]
}

fn empty_contributors() -> Vec<ContributorMsg> {
    vec![]
}

fn init_max_edition(deps: DepsMut, contributors: Vec<ContributorMsg>) {
    let msg = InstantiateMsg {
        base_token_uri: "https://ipfs.io/ipfs/".into(),
        collection_uri: "https://ipfs.io/ipfs/".into(),
        contributors: contributors,
        duration: Duration::Time(0),
        max_editions: 100,
        name: "name".into(),
        price: coin(100, "ubtsg"),
        referral_fee_bps: 0,
        seller_fee_bps: 0,
        start_time: Timestamp::from_seconds(0),
        symbol: "symbol".into(),
    };

    let info = mock_info("creator", &[]);
    instantiate(deps, mock_env(), info, msg).unwrap();
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    init_max_edition(deps.as_mut(), contributors());
}
#[test]
fn test_instantiate_duplicate_contributors() {
    let mut deps = mock_dependencies();
    init_max_edition(deps.as_mut(), duplicate_contributors());
}

#[test]
fn test_instantiate_empty_contributors() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        base_token_uri: "https://ipfs.io/ipfs/".into(),
        collection_uri: "https://ipfs.io/ipfs/".into(),
        contributors: empty_contributors(),
        duration: Duration::Time(0),
        max_editions: 100,
        name: "name".into(),
        price: coin(100, "ubtsg"),
        referral_fee_bps: 0,
        seller_fee_bps: 0,
        start_time: Timestamp::from_seconds(0),
        symbol: "symbol".into(),
    };

    let info = mock_info("creator", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::ContributorsEmpty {}) => {}
        _ => panic!("Unexpected error"),
    }
}

#[test]
fn test_query_get_config() {
    let mut deps = mock_dependencies();
    init_max_edition(deps.as_mut(), contributors());

    let config = query_get_config(deps.as_ref()).unwrap();
    assert_eq!(config.base_token_uri, "https://ipfs.io/ipfs/");
    assert_eq!(config.max_editions, 100);
    assert_eq!(config.name, "name");
    assert_eq!(config.price, coin(100, "ubtsg"));
    assert_eq!(config.referral_fee_bps, 0);
    assert_eq!(config.seller_fee_bps, 0);
    assert_eq!(config.start_time, Timestamp::from_seconds(0));
    assert_eq!(config.symbol, "symbol");
}

#[test]
fn test_execute_mint() {
    let mut deps = mock_dependencies();
    init_max_edition(deps.as_mut(), contributors());

    let info = mock_info("creator", &[]);
    let msg = ExecuteMsg::Mint {
        amount: Some(1),
        referral: None,
    };
    let res = crate::contract::execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Bs721NotLinked {}) => {}
        _ => panic!("Unexpected error"),
    }
}
