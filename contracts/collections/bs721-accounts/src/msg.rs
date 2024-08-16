use bs721::{Expiration, NftInfoResponse};
use bs721_base::msg::ExecuteMsg as Bs721ExecuteMsg;
use bs721_base::InstantiateMsg as Bs721InstantiateMsg;
use bs_account::{Metadata, TextRecord, NFT};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, CustomMsg, Empty};

use bs721::{
    AllNftInfoResponse, ApprovalResponse, ApprovalsResponse, ContractInfoResponse,
    NumTokensResponse, OperatorsResponse, OwnerOfResponse, TokensResponse,
};
use bs721_base::{MinterResponse, QueryMsg as Bs721QueryMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub verifier: Option<String>,
    pub marketplace: Addr,
    pub base_init_msg: Bs721InstantiateMsg,
}

// Add execute msgs related to metadata, image, text records
// The rest are inherited from sg721 and impl to properly convert the msgs.
#[cw_serde]
pub enum ExecuteMsg<T> {
    /// Set name marketplace contract address
    SetMarketplace { address: String },
    /// Set an address for name reverse lookup and updates token_uri
    /// Can be an EOA or a contract address.
    AssociateAddress {
        name: String,
        address: Option<String>,
    },
    /// Update image NFT
    UpdateImageNft { name: String, nft: Option<NFT> },
    /// Add text record ex: twitter handle, discord name, etc
    AddTextRecord { name: String, record: TextRecord },
    /// Remove text record ex: twitter handle, discord name, etc
    RemoveTextRecord { name: String, record_name: String },
    /// Update text record ex: twitter handle, discord name, etc
    UpdateTextRecord { name: String, record: TextRecord },
    /// Verify a text record as true or false (via oracle)
    VerifyTextRecord {
        name: String,
        record_name: String,
        result: bool,
    },
    /// Update the reset the verification oracle
    UpdateVerifier { verifier: Option<String> },
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
    /// Mint a new NFT, can only be called by the contract minter
    Mint {
        /// Unique ID of the NFT
        token_id: String,
        /// The owner of the newly minted NFT
        owner: String,
        /// Universal resource identifier for this NFT
        /// Should point to a JSON file that conforms to the ERC721
        /// Metadata JSON Schema
        token_uri: Option<String>,
        /// Seller fee basis points, 0-10000
        /// 0 means no fee, 100 means 1%, 10000 means 100%
        /// This is the fee paid by the buyer to the original creator
        seller_fee_bps: Option<u16>,
        /// Payment address, is the address that will receive the payment
        payment_addr: Option<String>,
        /// Any custom extension used by this contract
        extension: T,
    },
    /// Burn an NFT the sender has access to
    Burn { token_id: String },
    /// Freeze collection info from further updates
    FreezeCollectionInfo {},
}

impl<T> From<ExecuteMsg<T>> for Bs721ExecuteMsg<T> {
    fn from(msg: ExecuteMsg<T>) -> Bs721ExecuteMsg<T> {
        match msg {
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => Bs721ExecuteMsg::TransferNft {
                recipient,
                token_id,
            },
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => Bs721ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            },
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => Bs721ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            },
            ExecuteMsg::ApproveAll { operator, expires } => {
                Bs721ExecuteMsg::ApproveAll { operator, expires }
            }
            ExecuteMsg::Revoke { spender, token_id } => {
                Bs721ExecuteMsg::Revoke { spender, token_id }
            }
            ExecuteMsg::RevokeAll { operator } => Bs721ExecuteMsg::RevokeAll { operator },
            ExecuteMsg::Burn { token_id } => Bs721ExecuteMsg::Burn { token_id },
            ExecuteMsg::Mint {
                token_id,
                owner,
                token_uri,
                extension,
                seller_fee_bps,
                payment_addr,
            } => Bs721ExecuteMsg::Mint {
                token_id,
                owner,
                token_uri,
                extension,
                seller_fee_bps,
                payment_addr,
            },
            _ => unreachable!("Invalid ExecuteMsg"),
        }
    }
}

#[cw_serde]
pub enum SudoMsg {
    UpdateParams { max_record_count: u32 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum Bs721AccountsQueryMsg {
    /// Returns sudo params
    #[returns(SudoParams)]
    Params {},
    /// Reverse lookup of name for address
    #[returns(String)]
    Account { address: String },
    /// Returns the marketplace contract address
    #[returns(Addr)]
    AccountMarketplace {},
    /// Returns the associated address for a name
    #[returns(Addr)]
    AssociatedAddress { name: String },
    /// Returns the image NFT for a name
    #[returns(Option<NFT>)]
    ImageNFT { name: String },
    /// Returns the text records for a name
    #[returns(Vec<TextRecord>)]
    TextRecords { name: String },
    /// Returns if Twitter is verified for a name
    #[returns(bool)]
    IsTwitterVerified { name: String },
    /// Returns the verification oracle address
    #[returns(Option<String>)]
    Verifier {},
    /// Everything below is inherited from sg721
    #[returns(OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(ApprovalResponse)]
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    #[returns(ApprovalsResponse)]
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(OperatorsResponse)]
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(NumTokensResponse)]
    NumTokens {},
    #[returns(ContractInfoResponse)]
    ContractInfo {},
    #[returns(NftInfoResponse<Metadata>)]
    NftInfo { token_id: String },
    #[returns(AllNftInfoResponse<Metadata>)]
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(MinterResponse)]
    Minter {},
    // #[returns(CollectionInfoResponse)]
    // CollectionInfo {},
}

impl From<Bs721AccountsQueryMsg> for Bs721QueryMsg<Bs721AccountsQueryMsg> {
    fn from(msg: Bs721AccountsQueryMsg) -> Bs721QueryMsg<Bs721AccountsQueryMsg> {
        match msg {
            Bs721AccountsQueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => Bs721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            },
            Bs721AccountsQueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            } => Bs721QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            },
            Bs721AccountsQueryMsg::Approvals {
                token_id,
                include_expired,
            } => Bs721QueryMsg::Approvals {
                token_id,
                include_expired,
            },
            Bs721AccountsQueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            } => Bs721QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            },
            Bs721AccountsQueryMsg::NumTokens {} => Bs721QueryMsg::NumTokens {},
            Bs721AccountsQueryMsg::ContractInfo {} => Bs721QueryMsg::ContractInfo {},
            Bs721AccountsQueryMsg::NftInfo { token_id } => Bs721QueryMsg::NftInfo { token_id },
            Bs721AccountsQueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => Bs721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            },
            Bs721AccountsQueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Bs721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            Bs721AccountsQueryMsg::AllTokens { start_after, limit } => {
                Bs721QueryMsg::AllTokens { start_after, limit }
            }
            Bs721AccountsQueryMsg::Minter {} => Bs721QueryMsg::Minter {},
            // QueryMsg::CollectionInfo {} => Bs721QueryMsg::CollectionInfo {},
            _ => unreachable!("cannot convert {:?} to Cw721QueryMsg", msg),
        }
    }
}

// impl Default for Bs721AccountsQueryMsg {
//     fn default() -> Self {
//         Bs721AccountsQueryMsg::CheckRoyalties {}
//     }
// }

impl CustomMsg for Bs721AccountsQueryMsg {}

#[cosmwasm_schema::cw_serde]
pub struct SudoParams {
    pub max_record_count: u32,
}
