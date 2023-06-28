use cosmwasm_std::coins;

use crate::{ContractError, multitest::suite::{OWNER, DENOM}, msg::ContributorMsg};

use super::suite::TestSuiteBuilder;

pub const CONTRIBUTOR2: &str = "biz";

#[test]
pub fn distribute_fail() {
    let mut suite = TestSuiteBuilder::new().build();

    {
        let resp = suite.distribute(OWNER).unwrap_err();
        assert_eq!(
            ContractError::NothingToDistribute {},
            resp.downcast().unwrap(),
            "expected error since contract has no tokens"
        );
    }

    {
        suite.mint_to_contract(coins(1, "not_denom"));
        let resp = suite.distribute(OWNER).unwrap_err();
        assert_eq!(
            ContractError::NothingToDistribute {},
            resp.downcast().unwrap(),
            "expected error since contract has not tokens of the requried denom"
        );
    }

    {
        suite.mint_to_contract(coins(1, DENOM));
        let resp = suite.distribute(OWNER).unwrap_err();
        assert_eq!(
            ContractError::NothingToDistribute {},
            resp.downcast().unwrap(),
            "expected error since contract has not tokens of the requried denom"
        );
    }


}

#[test]
pub fn distribute_works() {
    let contributor1 = ContributorMsg {
        role: String::from("drawer"),
        shares: 1,
        address: String::from("drawer0000"),
    };

    let mut suite = TestSuiteBuilder::new().with_contributors(vec![contributor1]).build();

    {
        suite.mint_to_contract(coins(11, DENOM));
        let resp = suite.distribute(OWNER).unwrap();
    }
}
