use bs721_royalties::msg::ContributorMsg;
use cosmwasm_std::Addr;

use crate::msg::ConfigResponse;

use super::suite::TestSuiteBuilder;

#[test]
fn instantiate() {
    let contributors = vec![ContributorMsg {
        role: String::from("creator"),
        shares: 100,
        address: String::from("contributor0000"),
    }];
    let suite = TestSuiteBuilder::new()
        .with_contributors(contributors)
        .build();

    let resp = suite.query_config();

    // Ensure created contract addresses are correctly saved in the state
    assert_eq!(
        resp.bs721_base,
        Some(Addr::unchecked("contract1")),
        "expected bs721 base as second contract stored and saved in the state"
    );

    assert_eq!(
        resp.bs721_royalties,
        Some(Addr::unchecked("contract2")),
        "expected bs721 royalties as third contract stored and saved in the state"
    )
}

#[test]
fn mint() {
    let mut suite = TestSuiteBuilder::new()
        .with_default_contributors(vec![1, 2, 3])
        .build();

    suite.mint(None).unwrap();
}
