use bs721_royalties::msg::ContributorMsg;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, StdResult, Timestamp, Uint128};

use crate::ContractError;

/// Contributor info.
#[cw_serde]
pub struct Contributor {
    /// Address of the contributor.
    pub addr: String,
    pub weight: u64,
}

/// Possible launchparty type. Each type defines how the party end.
#[cw_serde]
pub enum PartyType {
    /// Maximum number of mintable tokens.
    MaxEdition(u32),
    /// Number of blocks for which tokens are mintable.
    Duration(u32),
}

#[cw_serde]
pub struct InstantiateMsg {
    /// Creator of the collection. If not provided it will be the sender.
    pub creator: Option<String>,
    /// BS721 token name.
    pub name: String,
    /// BS721 token symbol.
    pub symbol: String,
    /// Price of single nft minting.
    pub price: Coin,
    /// BS721 token uri.
    pub base_token_uri: String,
    /// BS721 collection uri.
    pub collection_uri: String,
    pub seller_fee_bps: u16,
    pub referral_fee_bps: u16,
    /// Contributors to the collection.
    pub contributors: Vec<ContributorMsg>,
    /// Start time of the launchparty.
    pub start_time: Timestamp,
    /// End condition of the collection launchparty.
    pub party_type: PartyType,
    /// Code id used to instantiate a bs721 token contract.
    pub bs721_token_code_id: u64,
    /// Code id used to instantiate bs721 royalties contract.
    pub bs721_royalties_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Retrieves contract's configuration
    #[returns(ConfigResponse)]
    GetConfig {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub creator: Addr,
    pub bs721_base: Option<Addr>,
    pub bs721_royalties: Option<Addr>,
    pub price: Coin,
    pub name: String,
    pub symbol: String,
    pub base_token_uri: String,
    pub next_token_id: u32,
    pub seller_fee_bps: u16,
    pub start_time: Timestamp,
    pub party_type: PartyType,
}

impl InstantiateMsg {
    /// Performs basic validation checks on the InstantiateMsg type.
    ///
    /// # Validation Checks:
    ///
    /// - maximum bps allowed for both seller and referral.
    /// - end condition of the launchparty.
    pub fn validate(&self) -> Result<(), ContractError> {
        // validate seller_fee_bps
        if self.seller_fee_bps > 10_000 {
            return Err(ContractError::FeeBps {
                profile: String::from("seller"),
            });
        }

        // validate referral_fee_bps
        if self.referral_fee_bps > 10_000 {
            return Err(ContractError::FeeBps {
                profile: String::from("referral"),
            });
        }

        self.party_type.validate()?;

        Ok(())
    }
}

impl PartyType {
    /// Performs basic validation checks on the party type.
    ///
    /// # Validation Checks
    ///
    /// - the number of maximum edition cannot be zero.
    /// - or, the party cannot end in the same time of the instantiation.
    pub fn validate(&self) -> Result<(), ContractError> {
        match self {
            PartyType::MaxEdition(number) => {
                if number == &0u32 {
                    return Err(ContractError::ZeroEditions {});
                }
            }
            PartyType::Duration(duration) => {
                if duration == &0u32 {
                    return Err(ContractError::ZeroDuration {});
                }
            }
        }
        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------
// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use cosmwasm_std::coin;

    use super::*;

    #[test]
    fn party_type_validate_works() {
        {
            let party_type = PartyType::Duration(0);
            let err = party_type.validate().unwrap_err();

            assert_eq!(
                err,
                ContractError::ZeroDuration {},
                "expected to fail since no zero duration party is allowed"
            );
        }

        {
            let party_type = PartyType::MaxEdition(0);
            let err = party_type.validate().unwrap_err();

            assert_eq!(
                err,
                ContractError::ZeroEditions {},
                "expected to fail since no party with zero editions is allowed"
            );
        }
    }

    #[test]
    fn instantiate_msg_validate_works() {
        let mut msg = InstantiateMsg {
            name: "Launchparty".to_string(),
            price: coin(1, "ubtsg"),
            creator: Some(String::from("creator")),
            symbol: "LP".to_string(),
            base_token_uri: "ipfs://Qm......".to_string(),
            collection_uri: "ipfs://Qm......".to_string(),
            seller_fee_bps: 100,
            referral_fee_bps: 1,
            contributors: vec![],
            start_time: Timestamp::from_seconds(0),
            party_type: PartyType::MaxEdition(1),
            bs721_royalties_code_id: 0,
            bs721_token_code_id: 1,
        };

        {
            msg.seller_fee_bps = 10_001;
            let err = msg.validate().unwrap_err();
            assert_eq!(
                err,
                ContractError::FeeBps {
                    profile: String::from("seller")
                },
                "expected to fail since fee bps higher than maximum allowed"
            );
            msg.seller_fee_bps = 1_000;
        }

        {
            msg.referral_fee_bps = 10_001;
            let err = msg.validate().unwrap_err();
            assert_eq!(
                err,
                ContractError::FeeBps {
                    profile: String::from("referral")
                },
                "expected to fail since fee bps higher than maximum allowed"
            );
            msg.referral_fee_bps = 1_000;
        }
    }
}
