use cosmwasm_std::{
    coin,
    testing::{mock_dependencies, mock_env, mock_info},
    BankMsg, CosmosMsg, Decimal, DepsMut,
};

use crate::contract::execute;
use crate::msg::ExecuteMsg;
use crate::{
    contract::{instantiate, query_list_contributors},
    msg::{ContributorListResponse, ContributorMsg, ContributorResponse, InstantiateMsg},
    ContractError,
};

const DENOM: &str = "ubtsg";
const CONTRIBUTOR1: &str = "contributor1";
const CONTRIBUTOR2: &str = "contributor2";
const CONTRIBUTOR3: &str = "contributor3";

fn init(deps: DepsMut) {
    let msg = InstantiateMsg {
        denom: DENOM.into(),
        contributors: vec![
            ContributorMsg {
                address: CONTRIBUTOR1.into(),
                role: "role".into(),
                shares: 10,
            },
            ContributorMsg {
                address: CONTRIBUTOR2.into(),
                role: "role".into(),
                shares: 20,
            },
            ContributorMsg {
                address: CONTRIBUTOR3.into(),
                role: "role".into(),
                shares: 30,
            },
        ],
    };

    let info = mock_info("creator", &[]);
    instantiate(deps, mock_env(), info, msg).unwrap();
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    init(deps.as_mut());
}

#[test]
fn test_query_list_contributors() {
    let mut deps = mock_dependencies();
    init(deps.as_mut());

    let contributors = query_list_contributors(deps.as_ref(), None, None).unwrap();
    assert_eq!(contributors.contributors.len(), 3);

    assert_eq!(
        contributors,
        ContributorListResponse {
            contributors: vec![
                ContributorResponse {
                    address: CONTRIBUTOR1.into(),
                    role: "role".into(),
                    initial_shares: 10,
                    percentage_shares: Decimal::from_ratio(10u128, 60u128)
                },
                ContributorResponse {
                    address: CONTRIBUTOR2.into(),
                    role: "role".into(),
                    initial_shares: 20,
                    percentage_shares: Decimal::from_ratio(20u128, 60u128)
                },
                ContributorResponse {
                    address: CONTRIBUTOR3.into(),
                    role: "role".into(),
                    initial_shares: 30,
                    percentage_shares: Decimal::from_ratio(30u128, 60u128)
                },
            ]
        }
    );

    let contributors =
        query_list_contributors(deps.as_ref(), Some(CONTRIBUTOR1.into()), None).unwrap();
    assert_eq!(contributors.contributors.len(), 2);

    let contributors =
        query_list_contributors(deps.as_ref(), Some(CONTRIBUTOR2.into()), None).unwrap();
    assert_eq!(contributors.contributors.len(), 1);

    let contributors =
        query_list_contributors(deps.as_ref(), Some(CONTRIBUTOR3.into()), None).unwrap();
    assert_eq!(contributors.contributors.len(), 0);

    let contributors =
        query_list_contributors(deps.as_ref(), Some(CONTRIBUTOR1.into()), Some(1)).unwrap();
    assert_eq!(contributors.contributors.len(), 1);

    assert_eq!(
        contributors,
        ContributorListResponse {
            contributors: vec![ContributorResponse {
                address: CONTRIBUTOR2.into(),
                role: "role".into(),
                initial_shares: 20,
                percentage_shares: Decimal::from_ratio(20u128, 60u128)
            }]
        }
    )
}

#[test]
fn test_execute_withdraw_for_all() {
    let env = mock_env();
    let mut deps = mock_dependencies();
    init(deps.as_mut());

    let msg = ExecuteMsg::WithdrawForAll {};

    let unauthorized_info = mock_info("unauthorized", &[]);
    let err_res = execute(deps.as_mut(), env.clone(), unauthorized_info, msg.clone());
    match err_res {
        Err(ContractError::Unauthorized {}) => {}
        _ => panic!("expected error"),
    }

    let info = mock_info(CONTRIBUTOR1, &[]);

    let no_fund_res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone());
    match no_fund_res {
        Err(ContractError::NoFunds {}) => {}
        _ => panic!("expected error"),
    }

    deps.querier.update_balance(
        env.contract.address.clone(),
        vec![coin(1000u128, DENOM.to_string())],
    );

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    assert_eq!(res.messages.len(), 3);

    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: CONTRIBUTOR1.into(),
            amount: vec![coin(330u128, DENOM.to_string())],
        })
    );

    assert_eq!(
        res.messages[1].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: CONTRIBUTOR2.into(),
            amount: vec![coin(330u128, DENOM.to_string())],
        })
    );

    assert_eq!(
        res.messages[2].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: CONTRIBUTOR3.into(),
            amount: vec![coin(330u128, DENOM.to_string())],
        })
    );
}
