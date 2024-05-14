// mod msg;
mod query;
mod receiver;
mod traits;

pub use cw_utils::Expiration;

pub use crate::query::{
    AllNftInfoResponse, Approval, ApprovalResponse, ApprovalsResponse, Bs721QueryMsg,
    ContractInfoResponse, NftInfoResponse, NumTokensResponse, OperatorsResponse, OwnerOfResponse,
    TokensResponse,
};
pub use crate::receiver::Bs721ReceiveMsg;
pub use crate::traits::{Bs721, Bs721Execute, Bs721Query};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Decimal, Timestamp};
// use cw_utils::Expiration;

#[cw_serde]
pub enum Bs721ExecuteMsg {
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke { spender: String, token_id: String },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },
    /// Burn an NFT the sender has access to
    Burn { token_id: String },
    /// Update specific collection info fields
    UpdateCollectionInfo {
        collection_info: UpdateCollectionInfoMsg<RoyaltyInfoResponse>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct RoyaltyInfo {
    pub payment_address: Addr,
    pub payment_denom: String,
    pub share: Decimal,
}
// allows easy conversion from RoyaltyInfo to RoyaltyInfoResponse
impl RoyaltyInfo {
    pub fn to_response(&self) -> RoyaltyInfoResponse {
        RoyaltyInfoResponse {
            payment_address: self.payment_address.to_string(),
            payment_denom: self.payment_denom.to_string(),
            share: self.share,
        }
    }
}

#[cosmwasm_schema::cw_serde]
#[derive(Default)]
pub struct RoyaltyInfoResponse {
    pub payment_address: String,
    pub payment_denom: String,
    pub share: Decimal,
}

#[cosmwasm_schema::cw_serde]
pub struct CollectionInfo<T> {
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub explicit_content: Option<bool>,
    pub start_trading_time: Option<Timestamp>,
    pub royalty_info: Option<T>,
}

impl<T> Default for CollectionInfo<T>
where
    T: Default,
{
    fn default() -> Self {
        CollectionInfo {
            creator: "creator".to_string(),
            description: String::new(),
            image: "https://www.beautiful.network".to_string(),
            external_link: Some("https://www.beautiful.network".to_string()),
            explicit_content: None,
            start_trading_time: None,
            royalty_info: Default::default(),
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct UpdateCollectionInfoMsg<T> {
    pub description: Option<String>,
    pub image: Option<String>,
    pub external_link: Option<Option<String>>,
    pub explicit_content: Option<bool>,
    pub royalty_info: Option<Option<T>>,
    pub creator: Option<String>,
}
