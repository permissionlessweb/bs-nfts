use bs721_royalties::msg::ContributorMsg;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Env, Timestamp};

use bs721_curve::msg::Metadata as Bs721CurveMetadata;

use launchparty_fixed::msg::{Metadata as LaunchpartyFixedMetadata, PartyType};

use crate::{state::Config, ContractError};

#[cw_serde]
pub struct InstantiateMsg {
    /// Address allowed to update contract parameters
    pub owner: String,
    /// Code id used to instantiate a bs721 metadata onchain token contract.
    pub bs721_metadata_code_id: u64,
    /// Code id used to instantiate bs721 royalties contract. The address of this contract will be used
    /// as the payment address for the NFT mint.
    pub bs721_royalties_code_id: u64,

    pub bs721_simple_sale_code_id: u64,

    pub bs721_curve_sale_code_id: u64,
    /// Protocol fee as basis points
    pub protocol_fee_bps: u32,

    pub create_nft_sale_fee: Coin,
}

#[cw_serde]
pub struct MsgCreateNftSimpleSale {
    /// BS721 token symbol.
    pub symbol: String,
    /// Price of single nft minting.
    pub price: Coin,
    /// Maximum amount of tokens an address can mint.
    pub max_per_address: Option<u32>,
    /// BS721 collection image.
    pub collection_image: String,
    /// BS721 collection cover image.
    pub collection_cover_image: Option<String>,
    /// On-chain Metadata
    pub metadata: LaunchpartyFixedMetadata,
    /// Basis per point of the `price` sent to the referred address during mint. This payment is sent
    /// one-off.
    pub seller_fee_bps: u16,
    /// Basis per point of the `price` sent to the referred address during mint. This payment is sent
    /// one-off.
    pub referral_fee_bps: u16,
    /// Contributors to the collection.
    pub contributors: Vec<ContributorMsg>,
    /// Start time of the launchparty.
    pub start_time: Timestamp,
    /// End condition of the collection launchparty.
    pub party_type: PartyType,
}

#[cw_serde]
pub struct MsgCreateNftCurveSale {
    /// BS721 token symbol.
    pub symbol: String,
    /// Denom used to pay for the NFTs
    pub payment_denom: String,
    /// Maximum amount of tokens an address can mint.
    pub max_per_address: Option<u32>,
    /// BS721 collection image.
    pub collection_image: String,
    /// BS721 collection cover image.
    pub collection_cover_image: Option<String>,
    /// On-chain Metadata
    pub metadata: Bs721CurveMetadata,
    /// Basis per point of the `price` sent to the referred address during mint. This payment is sent
    /// one-off.
    pub seller_fee_bps: u16,
    /// Basis per point of the `price` sent to the referred address during mint. This payment is sent
    /// one-off.
    pub referral_fee_bps: u16,
    /// Contributors to the collection.
    pub contributors: Vec<ContributorMsg>,
    /// Start time of the launchparty.
    pub start_time: Timestamp,
    /// Max edition of the collection launchparty.
    pub max_edition: Option<u32>,
    /// Ratio, is the cooeficient of the curve
    pub ratio: u32,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        /// Address allowed to update contract parameters
        owner: Option<String>,
        /// Code id used to instantiate a bs721 metadata onchain token contract.
        bs721_metadata_code_id: Option<u64>,
        /// Code id used to instantiate bs721 royalties contract. The address of this contract will be used
        /// as the payment address for the NFT mint.
        bs721_royalties_code_id: Option<u64>,

        bs721_simple_sale_code_id: Option<u64>,

        bs721_curve_sale_code_id: Option<u64>,
        /// Protocol fee as basis points
        protocol_fee_bps: Option<u32>,

        create_nft_sale_fee: Option<Coin>,
    },

    CreateNftSimpleSale(MsgCreateNftSimpleSale),

    CreateNftCurveSale(MsgCreateNftCurveSale),
}

/// Possible query messages that the launchparty-curve contract can handle.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Retrieves contract's configuration
    #[returns(Config)]
    Config {},
}

impl InstantiateMsg {
    const MAX_FEE_BPS: u32 = 10_000;

    pub fn validate(&self, _env: Env) -> Result<(), ContractError> {
        // validate referral_fee_bps
        if self.protocol_fee_bps > Self::MAX_FEE_BPS {
            return Err(ContractError::FeeBps {
                profile: String::from("referral"),
            });
        }

        Ok(())
    }
}
