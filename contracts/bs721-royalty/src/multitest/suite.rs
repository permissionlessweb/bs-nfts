use anyhow::Result as AnyResult;
use cosmwasm_std::{Addr, Coin, Empty, Uint128};
use cw_multi_test::{App, AppResponse, BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
use derivative::Derivative;

use crate::{
    contract,
    msg::{ContributorListResponse, ExecuteMsg, InstantiateMsg, QueryMsg},
    msg::{ContributorMsg, ContributorResponse},
};

pub const OWNER: &str = "owner";
pub const DENOM: &str = "bitsong";
pub const DEFAULT_SHARE: u32 = 10;

/// bs721-royalties contract.
fn royalty_contract() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new_with_empty(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    ))
}

fn instantiate_royalty(
    app: &mut App,
    code_id: u64,
    denom: String,
    contributors: Vec<ContributorMsg>,
) -> Addr {
    app.instantiate_contract(
        code_id,
        Addr::unchecked(String::from(OWNER)),
        &InstantiateMsg {
            denom,
            contributors,
        },
        &vec![],
        "Bitsong royalties contract",
        None,
    )
    .unwrap()
}

/// Stores genesis configuration for tests. This strcture is used initialize a TestSuite.
#[derive(Derivative, Debug)]
#[derivative(Default)]
pub struct TestSuiteBuilder {
    #[derivative(Default(value = "String::from(OWNER)"))]
    contracts_owner: String,
    // Royalties denom.
    #[derivative(Default(value = "String::from(DENOM)"))]
    denom: String,
    /// Initial contributors.
    contributors: Vec<ContributorMsg>,
}

impl TestSuiteBuilder {
    /// Constructor with default values.
    pub fn new() -> Self {
        Self {
            contributors: vec![ContributorMsg {
                role: String::from("creator"),
                share: DEFAULT_SHARE,
                address: String::from(OWNER),
            }],
            ..Default::default()
        }
    }

    /// Change roylaties denom.
    pub fn with_denom(mut self, denom: String) -> Self {
        self.denom = denom;
        self
    }

    /// Append `contributors` to the initial vector of contributors.
    pub fn with_contributors(mut self, contributors: Vec<ContributorMsg>) -> Self {
        self.contributors = [self.contributors, contributors].concat();
        self
    }

    pub fn build(self) -> TestSuite {
        let mut app = App::default();

        let code_id = app.store_code(royalty_contract());
        let contract_address =
            instantiate_royalty(&mut app, code_id, self.denom, self.contributors);

        TestSuite {
            app,
            contract_address,
        }
    }
}

pub struct TestSuite {
    app: App,
    contract_address: Addr,
}

impl TestSuite {
    /// Helper to mint `amount` to the royalty contract.
    pub fn mint_to_contract(&mut self, amount: Vec<Coin>) {
        self.app
            .sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: self.contract_address.to_string(),
                amount,
            }))
            .unwrap();
    }

    /// Helper distribute royalty shares to contributors.
    pub fn distribute(&mut self, sender: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract_address.clone(),
            &ExecuteMsg::Distribute {},
            &vec![],
        )
    }

    pub fn query_contributors(
        &self,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> ContributorListResponse {
        self.app
            .wrap()
            .query_wasm_smart(
                self.contract_address.clone(),
                &QueryMsg::ListContributors { start_after, limit },
            )
            .unwrap()
    }

    pub fn query_withdrawable_amount(
        &self,
    ) -> Uint128 {
        self.app
            .wrap()
            .query_wasm_smart(
                self.contract_address.clone(),
                &QueryMsg::WithdrawableAmount {  },
            )
            .unwrap()
    }
}
