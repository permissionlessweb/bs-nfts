use cosmwasm_schema::{cw_serde, QueryResponses};
use crate::ContractError;

#[cw_serde]
pub struct ContributorMsg {
    pub role: String,
    pub share: u32,
    pub address: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
    pub contributors: Vec<ContributorMsg>,
}

impl InstantiateMsg {
    pub fn validate(&mut self) -> Result<(), ContractError> {
        if self.contributors.clone().is_empty() {
            return Err(ContractError::EmptyContributors {});
        }

        let mut total_share = 0;
        for contributor in &self.contributors {
            total_share += contributor.share;
        }
        if total_share == 0 {
            return Err(ContractError::InvalidShares {});
        }

        // validate unique contributors
        self.contributors.sort_by(|a, b| a.address.cmp(&b.address));
        for (a, b) in self.contributors.iter().zip(self.contributors.iter().skip(1)) {
            if a.address == b.address {
                return Err(ContractError::DuplicateContributor {
                    contributor: a.address.clone(),
                });
            }
        }

        Ok(())
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    WithdrawForAll { },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ContributorListResponse)]
    ListContributors {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ContributorListResponse {
    pub contributors: Vec<ContributorResponse>,
}

#[cw_serde]
pub struct ContributorResponse {
    pub role: String,
    pub share: u32,
    pub address: String,
}
