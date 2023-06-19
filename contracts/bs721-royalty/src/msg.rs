use crate::ContractError;
use cosmwasm_schema::{cw_serde, QueryResponses};

/// Represents a contributor to the collection.
#[cw_serde]
pub struct ContributorMsg {
    /// Contributor's role
    pub role: String,
    /// Amount of share associated to the contributor.
    pub share: u32,
    /// Contributor's address.
    pub address: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    /// Native denom distributed to contributors.
    pub denom: String,
    /// NFT collection contibutors.
    pub contributors: Vec<ContributorMsg>,
}

impl InstantiateMsg {
    pub fn validate(&mut self) -> Result<(), ContractError> {
        // cannot instantiate a royality contract without at least one contributor
        if self.contributors.clone().is_empty() {
            return Err(ContractError::EmptyContributors {});
        }

        for contributor in &self.contributors {
            if contributor.share == 0 {
                return Err(ContractError::InvalidShares {});
            }
        }

        // validate unique contributors
        self.contributors.sort_by(|a, b| a.address.cmp(&b.address));
        for (a, b) in self
            .contributors
            .iter()
            .zip(self.contributors.iter().skip(1))
        {
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
    /// Withdraw royalties for each contributor. This message can only be sent by a contributor.
    WithdrawForAll {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Retrieves the list of contributors.
    #[returns(ContributorListResponse)]
    ListContributors {
        /// Address after which contributors are retrieved.
        start_after: Option<String>,
        /// Number of contributors to receive.
        limit: Option<u32>,
    },
}

/// Retrieved contributors response
#[cw_serde]
pub struct ContributorListResponse {
    pub contributors: Vec<ContributorResponse>,
}

/// Single contributor response info.
#[cw_serde]
pub struct ContributorResponse {
    /// Role of the contributor.
    pub role: String,
    /// Shares of the contributor.
    pub share: u32,
    /// Role of the contributor.
    pub address: String,
}

// -------------------------------------------------------------------------------------------------
// Unit tests
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::ContractError;

    use crate::InstantiateMsg;

    use super::ContributorMsg;

    #[test]
    pub fn validate_single() {
        let mut msg = InstantiateMsg {
            denom: "bitsong".to_owned(),
            contributors: vec![],
        };

        {
            let val = msg.validate().unwrap_err();
            assert_eq!(
                val,
                ContractError::EmptyContributors {},
                "expected to fail since at least one contributor is requried"
            )
        }

        {
            let contributor = ContributorMsg {
                role: String::from("dj"),
                share: 0,
                address: String::from("bitsong0000"),
            };

            msg.contributors.push(contributor);

            let val = msg.validate().unwrap_err();
            assert_eq!(
                val,
                ContractError::InvalidShares {},
                "expected to fail since zero shares is not allowed"
            )
        }
        {
            let contributor = ContributorMsg {
                role: String::from("dj"),
                share: 10,
                address: String::from("bitsong0000"),
            };

            msg = InstantiateMsg {
                contributors: vec![contributor],
                ..msg
            };

            let val = msg.validate().unwrap();
            assert_eq!(val, (), "expected to pass since valid msg")
        }
    }

    #[test]
    pub fn validate_multiple() {
        let contributor_1 = ContributorMsg {
            role: String::from("dj"),
            share: 10,
            address: String::from("bitsong0000"),
        };

        {
            let contributor_2 = ContributorMsg {
                role: String::from("dj"),
                share: 10,
                address: String::from("bitsong0000"),
            };

            let mut msg = InstantiateMsg {
                denom: "bitsong".to_owned(),
                contributors: vec![contributor_1.clone(), contributor_2],
            };

            let val = msg.validate().unwrap_err();
            assert_eq!(
                val,
                ContractError::DuplicateContributor {
                    contributor: String::from("bitsong0000")
                },
                "expected to fail since zero shares is not allowed"
            )
        }

        {
            let contributor_2 = ContributorMsg {
                role: String::from("drawer"),
                share: 0,
                address: String::from("bitsong1111"),
            };

            let mut msg = InstantiateMsg {
                denom: "bitsong".to_owned(),
                contributors: vec![contributor_1.clone(), contributor_2],
            };

            let val = msg.validate().unwrap_err();
            assert_eq!(
                val,
                ContractError::InvalidShares {},
                "expected to fail since all contributors must have shares"
            )
        }

        {
            let contributor_2 = ContributorMsg {
                role: String::from("drawer"),
                share: 10,
                address: String::from("bitsong1111"),
            };

            let mut msg = InstantiateMsg {
                denom: "bitsong".to_owned(),
                contributors: vec![contributor_1, contributor_2],
            };

            let val = msg.validate().unwrap();
            assert_eq!(val, (), "expected to pass")
        }
    }
}
