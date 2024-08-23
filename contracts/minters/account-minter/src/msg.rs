use btsg_account::minter::{Config, SudoParams};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// Temporary admin for managing whitelists
    pub admin: Option<String>,
    /// Oracle for verifying text records
    pub verifier: Option<String>,
    /// Code-id for BS721-Account. On Instantiate, minter will instantiate a new account collection.
    pub collection_code_id: u64,
    /// Contract Address to Marketplace contract.
    pub marketplace_addr: String,
    /// Minimum length an account id can be
    pub min_name_length: u32,
    /// Maximum length an account id can be
    pub max_name_length: u32,
    /// Base price for a name. Used to calculate premium for small account names
    pub base_price: Uint128,
}

#[cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    /// Mint a account and list on Stargaze Account Marketplace
    MintAndList { account: String },
    /// Change the admin that manages the whitelist
    /// Will be set to null after go-to-market
    UpdateAdmin { admin: Option<String> },
    /// Admin can pause minting during whitelist switching
    Pause { pause: bool },
    /// Update config, only callable by admin
    UpdateConfig { config: Config },
}

#[cw_serde]
pub enum SudoMsg {
    UpdateParams {
        min_name_length: u32,
        max_name_length: u32,
        base_price: Uint128,
    },
    UpdateAccountCollection {
        collection: String,
    },
    UpdateAccountMarketplace {
        marketplace: String,
    },
}

#[cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    #[returns(cw_controllers::AdminResponse)]
    Admin {},
    #[returns(Addr)]
    Collection {},
    #[returns(SudoParams)]
    Params {},
    #[returns(Config)]
    Config {},
}
