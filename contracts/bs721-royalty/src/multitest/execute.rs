use cosmwasm_std::coins;

use crate::{ContractError, multitest::suite::{OWNER, DENOM}};

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
        _ = suite.distribute(OWNER).unwrap();
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
    let mut suite = TestSuiteBuilder::new().build();

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
