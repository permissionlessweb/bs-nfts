use cosmwasm_std::Uint128;

use crate::msg::{ContributorMsg, ContributorResponse};

use super::suite::TestSuiteBuilder;

#[test]
pub fn single_contributor() {
    let suite = TestSuiteBuilder::new().build();

    let resp = suite.query_contributors(None, None);

    assert_eq!(
        resp.contributors.len(),
        1,
        "expected only default contributor"
    )
}

#[test]
pub fn multiple_contributors() {
    let contributors = vec![
        ContributorMsg {
            role: String::from("drawer"),
            share: 10,
            address: String::from("drawer0000"),
        },
        ContributorMsg {
            role: String::from("biz"),
            share: 10,
            address: String::from("biz0000"),
        },
        ContributorMsg {
            role: String::from("marketer"),
            share: 10,
            address: String::from("marketer0000"),
        },
    ];

    let suite = TestSuiteBuilder::new()
        .with_contributors(contributors)
        .build();

    let resp = suite.query_contributors(None, None);
    assert_eq!(resp.contributors.len(), 4, "expected 4 contributors");
    assert_eq!(
        resp.contributors[1],
        ContributorResponse {
            role: String::from("drawer"),
            share: 10,
            address: String::from("drawer0000")
        }
    )
}

#[test]
pub fn withdrawable_amount() {
    let mut suite = TestSuiteBuilder::new().build();

    let resp = suite.query_withdrawable_amount();
    assert_eq!(Uint128::zero(), resp, "expected nothing to withdraw since never distributed");
}
