use bs721_base::msg::InstantiateMsg as Bs721InstantiateMsg;
use bs721_royalty::msg::{ContributorMsg, InstantiateMsg as Bs721RoyaltyInstantiateMsg};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_binary, Addr, Coin, Timestamp, WasmMsg};
use cw_utils::Duration;

use crate::ContractError;

const BS721_CODE_ID: u64 = 6;
const BS721_ROYALTY_CODE_ID: u64 = 8;

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub max_editions: u32,
    pub price: Coin,
    pub base_token_uri: String,
    pub collection_uri: String,
    pub seller_fee_bps: u16,
    pub referral_fee_bps: u16,
    pub contributors: Vec<ContributorMsg>,
    pub start_time: Timestamp,
    pub duration: Duration,
}

impl InstantiateMsg {
    pub fn validate(&self) -> Result<(), ContractError> {
        // validate duration and max_editions
        match self.duration {
            Duration::Height(height) => {
                if (height != 0 && self.max_editions != 0)
                    || (height == 0 && self.max_editions == 0)
                {
                    return Err(ContractError::DurationOrMaxEditions {});
                }
            }
            Duration::Time(time) => {
                if (time != 0 && self.max_editions != 0) || (time == 0 && self.max_editions == 0) {
                    return Err(ContractError::DurationOrMaxEditions {});
                }
            }
        }

        // validate seller_fee_bps
        if self.seller_fee_bps > 10000 {
            return Err(ContractError::SellerFeeBps {});
        }

        // validate reseller_fee_bps
        if self.referral_fee_bps > 10000 {
            return Err(ContractError::ReferralFeeBps {});
        }

        // validate contributors
        if self.contributors.is_empty() {
            return Err(ContractError::ContributorsEmpty {});
        }

        if self.contributors.len() > 100 {
            return Err(ContractError::ContributorsTooMany {});
        }

        Ok(())
    }

    pub fn into_bs721_wasm_msg(self, minter: String) -> WasmMsg {
        WasmMsg::Instantiate {
            code_id: BS721_CODE_ID,
            msg: to_binary(&Bs721InstantiateMsg {
                name: self.name,
                symbol: self.symbol,
                minter: minter,
                uri: Some(self.collection_uri),
            })
            .unwrap(),
            label: "bs721-drop-simple".to_string(),
            admin: None,
            funds: vec![],
        }
    }

    pub fn into_bs721_royalty_wasm_msg(self) -> WasmMsg {
        WasmMsg::Instantiate {
            code_id: BS721_ROYALTY_CODE_ID,
            msg: to_binary(&Bs721RoyaltyInstantiateMsg {
                denom: self.price.denom,
                contributors: self.contributors,
            })
            .unwrap(),
            label: "bs721-drop-simple: royalty contract".to_string(),
            admin: None,
            funds: vec![],
        }
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint {
        amount: Option<u32>,
        referral: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    GetConfig {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub creator: Addr,
    pub bs721_address: Option<Addr>,
    pub max_editions: u32,
    pub price: Coin,
    pub name: String,
    pub symbol: String,
    pub base_token_uri: String,
    pub next_token_id: u32,
    pub seller_fee_bps: u16,
    pub royalty_address: Option<Addr>,
    pub start_time: Timestamp,
    pub referral_fee_bps: u16,
    pub duration: Duration,
}
