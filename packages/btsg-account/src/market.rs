//! # Profile Marketplace
//!
//! `abstract_std::account_marketplace`
//!
//! ## Description

pub mod state {
    use bs_controllers::Hooks;
    use cosmwasm_std::{Addr, Decimal, StdResult, Storage, Timestamp, Uint128};
    use cw_storage_macro::index_list;
    use cw_storage_plus::{IndexedMap, Item, Map, MultiIndex, UniqueIndex};

    // bps fee can not exceed 100%
    pub const MAX_FEE_BPS: u64 = 10000;

    #[cosmwasm_schema::cw_serde]
    pub struct SudoParams {
        /// Fair Burn + Community Pool fee for winning bids
        pub trading_fee_percent: Decimal,
        /// Min value for a bid
        pub min_price: Uint128,
        /// Interval to rate limit setting asks (in seconds)
        pub ask_interval: u64,
        /// The maximum number of renewals that can be processed in each block
        pub max_renewals_per_block: u32,
        /// The number of bids to query to when searching for the highest bid
        pub valid_bid_query_limit: u32,
        /// The number of seconds before the current block time that a
        /// bid must have been created in order to be considered valid
        pub renew_window: u64,
        /// The percentage of the winning bid that must be paid to renew a name
        pub renewal_bid_percentage: Decimal,
        /// The address with permission to invoke process_renewals
        pub operator: Addr,
    }

    pub struct ParamInfo {
        pub trading_fee_bps: Option<u64>,
        pub min_price: Option<Uint128>,
        pub ask_interval: Option<u64>,
    }

    pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sp");

    pub const ASK_HOOKS: Hooks = Hooks::new("ah");
    pub const BID_HOOKS: Hooks = Hooks::new("bh");
    pub const SALE_HOOKS: Hooks = Hooks::new("sh");

    pub const ACCOUNT_MINTER: Item<Addr> = Item::new("am");
    pub const ACCOUNT_COLLECTION: Item<Addr> = Item::new("ac");
    pub const VERSION_CONTROL: Item<Addr> = Item::new("vc");

    /// (renewal_time (in seconds), id) -> [token_id]
    pub const RENEWAL_QUEUE: Map<(u64, u64), TokenId> = Map::new("rq");
    /// (new_owner, manager_addr) -> [governance_details]

    pub const ASK_COUNT: Item<u64> = Item::new("ask-count");
    pub const IS_SETUP: Item<bool> = Item::new("is");

    pub fn ask_count(storage: &dyn Storage) -> StdResult<u64> {
        Ok(ASK_COUNT.may_load(storage)?.unwrap_or_default())
    }

    pub fn increment_asks(storage: &mut dyn Storage) -> StdResult<u64> {
        let val = ask_count(storage)? + 1;
        ASK_COUNT.save(storage, &val)?;
        Ok(val)
    }

    pub fn decrement_asks(storage: &mut dyn Storage) -> StdResult<u64> {
        let val = ask_count(storage)? - 1;
        ASK_COUNT.save(storage, &val)?;
        Ok(val)
    }

    /// Type for storing the `ask`
    pub type TokenId = String;

    /// Type for `ask` unique secondary index
    pub type Id = u64;

    /// Represents an ask on the marketplace
    #[cosmwasm_schema::cw_serde]
    pub struct Ask {
        pub token_id: TokenId,
        pub id: u64,
        pub seller: Addr,
        pub renewal_time: Timestamp,
        pub renewal_fund: Uint128,
    }

    /// Primary key for asks: token_id
    /// Name reverse lookup can happen in O(1) time
    pub type AskKey = TokenId;
    /// Convenience ask key constructor
    pub fn ask_key(token_id: &str) -> AskKey {
        token_id.to_string()
    }

    /// Defines indices for accessing Asks
    #[index_list(Ask)]
    pub struct AskIndicies<'a> {
        /// Unique incrementing id for each ask
        /// This allows pagination when `token_id`s are strings
        pub id: UniqueIndex<'a, u64, Ask, AskKey>,
        /// Index by seller
        pub seller: MultiIndex<'a, Addr, Ask, AskKey>,
    }

    pub fn asks<'a>() -> IndexedMap<'a, AskKey, Ask, AskIndicies<'a>> {
        let indexes = AskIndicies {
            id: UniqueIndex::new(|d| d.id, "ask__id"),
            seller: MultiIndex::new(
                |_pk: &[u8], d: &Ask| d.seller.clone(),
                "asks",
                "asks__seller",
            ),
        };
        IndexedMap::new("asks", indexes)
    }

    /// Represents a bid (offer) on the marketplace
    #[cosmwasm_schema::cw_serde]
    pub struct Bid {
        pub token_id: TokenId,
        pub bidder: Addr,
        pub amount: Uint128,
        pub created_time: Timestamp,
    }

    impl Bid {
        pub fn new(token_id: &str, bidder: Addr, amount: Uint128, created_time: Timestamp) -> Self {
            Bid {
                token_id: token_id.to_string(),
                bidder,
                amount,
                created_time,
            }
        }
    }

    /// Primary key for bids: (token_id, bidder)
    pub type BidKey = (TokenId, Addr);
    /// Convenience bid key constructor
    pub fn bid_key(token_id: &str, bidder: &Addr) -> BidKey {
        (token_id.to_string(), bidder.clone())
    }

    /// Defines indices for accessing bids
    #[index_list(Bid)]
    pub struct BidIndicies<'a> {
        pub price: MultiIndex<'a, u128, Bid, BidKey>,
        pub bidder: MultiIndex<'a, Addr, Bid, BidKey>,
        pub created_time: MultiIndex<'a, u64, Bid, BidKey>,
    }

    pub fn bids<'a>() -> IndexedMap<'a, BidKey, Bid, BidIndicies<'a>> {
        let indexes = BidIndicies {
            price: MultiIndex::new(
                |_pk: &[u8], d: &Bid| d.amount.u128(),
                "bids",
                "bids__collection_price",
            ),
            bidder: MultiIndex::new(
                |_pk: &[u8], d: &Bid| d.bidder.clone(),
                "bids",
                "bids__bidder",
            ),
            created_time: MultiIndex::new(
                |_pk: &[u8], d: &Bid| d.created_time.seconds(),
                "bids",
                "bids__time",
            ),
        };
        IndexedMap::new("bids", indexes)
    }
}

use self::state::{Ask, Bid, Id, SudoParams, TokenId};
use bs_controllers::HooksResponse;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{to_json_binary, Addr, Binary, Coin, Decimal, StdResult, Uint128};

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Community pool fee for winning bids
    /// 0.25% = 25, 0.5% = 50, 1% = 100, 2.5% = 250
    pub trading_fee_bps: u64,
    /// Min value for bids and asks
    pub min_price: Uint128,
    /// Interval to rate limit setting asks (in seconds)
    pub ask_interval: u64,
    /// Account Factory contract address
    // pub factory: Addr,
    /// Account Profile contract address
    // pub collection: Addr,
    /// The maximum number of renewals that can be processed in each block
    pub max_renewals_per_block: u32,
    /// The number of bids to query to when searching for the highest bid
    pub valid_bid_query_limit: u32,
    /// The number of seconds before the current block time that a
    /// bid must have been created in order to be considered valid.
    /// Also, the number of seconds before an ask expires where it can be renewed.
    pub renew_window: u64,
    /// The percentage of the winning bid that must be paid to renew a name
    pub renewal_bid_percentage: Decimal,
    /// The address with permission to invoke process_renewals
    pub operator: String,
}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    /// List name NFT on the marketplace by creating a new ask.
    /// Only the account factory can call this.
    SetAsk { token_id: TokenId, seller: String },
    /// Remove account on the marketplace.
    /// Only the account collection can call this (i.e: when burned).
    RemoveAsk { token_id: TokenId },
    /// Update ask when an NFT is transferred
    /// Only the name collection can call this
    UpdateAsk { token_id: TokenId, seller: String },
    /// Place a bid on an existing ask
    SetBid { token_id: TokenId },
    /// Remove an existing bid from an ask
    RemoveBid { token_id: TokenId },
    /// Accept a bid on an existing ask
    AcceptBid { token_id: TokenId, bidder: String },
    /// Fully renew a name if within the renewal period
    // Renew { token_id: TokenId },
    /// Fund renewal of a name
    // FundRenewal { token_id: TokenId },
    /// Refund a renewal of a name
    // RefundRenewal { token_id: TokenId },
    /// Check if expired names have been paid for, and collect fees.
    /// If not paid, transfer ownership to the highest bidder.
    // ProcessRenewals { time: Timestamp },
    /// Can only be run once
    Setup { minter: String, collection: String },
}

#[cosmwasm_schema::cw_serde]
pub enum SudoMsg {
    /// Update the contract parameters
    /// Can only be called by governance
    UpdateParams {
        trading_fee_bps: Option<u64>,
        min_price: Option<Uint128>,
        ask_interval: Option<u64>,
    },
    /// Update the contract address of the account factory
    UpdateAccountFactory { factory: String },
    /// Update the contract address of the name collection
    UpdateAccountCollection { collection: String },
    /// Add a new hook to be informed of all asks
    AddAskHook { hook: String },
    /// Remove a ask hook
    RemoveAskHook { hook: String },
    /// Add a new hook to be informed of all bids
    AddBidHook { hook: String },
    /// Remove a bid hook
    RemoveBidHook { hook: String },
    /// Add a new hook to be informed of all trades
    AddSaleHook { hook: String },
    /// Remove a trade hook
    RemoveSaleHook { hook: String },
}

pub type Collection = String;
pub type Bidder = String;
pub type Seller = String;

/// Offset for ask pagination
#[cosmwasm_schema::cw_serde]
pub struct AskOffset {
    pub price: Uint128,
    pub token_id: TokenId,
}

impl AskOffset {
    pub fn new(price: Uint128, token_id: TokenId) -> Self {
        AskOffset { price, token_id }
    }
}

/// Offset for bid pagination
#[cosmwasm_schema::cw_serde]
pub struct BidOffset {
    pub price: Uint128,
    pub token_id: TokenId,
    pub bidder: Addr,
}

impl BidOffset {
    pub fn new(price: Uint128, token_id: TokenId, bidder: Addr) -> Self {
        BidOffset {
            price,
            token_id,
            bidder,
        }
    }
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    /// Get the current ask for specific name
    #[returns(Option<Ask>)]
    Ask { token_id: TokenId },
    /// Get all asks for a collection
    #[returns(Vec<Ask>)]
    Asks {
        start_after: Option<Id>,
        limit: Option<u32>,
    },
    /// Count of all asks
    #[returns(u64)]
    AskCount {},
    /// Get all asks by seller
    #[returns(Vec<Ask>)]
    AsksBySeller {
        seller: Seller,
        start_after: Option<TokenId>,
        limit: Option<u32>,
    },
    /// Get all renewable Asks
    // #[returns(Vec<Ask>)]
    // AsksByRenewTime {
    //     max_time: Timestamp,
    //     start_after: Option<Timestamp>,
    //     limit: Option<u32>,
    // },
    // /// Get the renewal price for a specific name
    // #[returns((Option<Coin>, Option<Bid>))]
    // AskRenewPrice {
    //     current_time: Timestamp,
    //     token_id: TokenId,
    // },
    // /// Get renewal price for multiple names
    // #[returns(Vec<AskRenewPriceResponse>)]
    // AskRenewalPrices {
    //     current_time: Timestamp,
    //     token_ids: Vec<TokenId>,
    // },
    /// Get data for a specific bid
    #[returns(Option<Bid>)]
    Bid { token_id: TokenId, bidder: Bidder },
    /// Get all bids by a bidder
    #[returns(Vec<Bid>)]
    BidsByBidder {
        bidder: Bidder,
        start_after: Option<TokenId>,
        limit: Option<u32>,
    },
    /// Get all bids for a specific NFT
    #[returns(Vec<Bid>)]
    Bids {
        token_id: TokenId,
        start_after: Option<Bidder>,
        limit: Option<u32>,
    },
    /// Get all bids for a collection, sorted by price
    #[returns(Vec<Bid>)]
    BidsSortedByPrice {
        start_after: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Get all bids for a collection, sorted by price in reverse
    #[returns(Vec<Bid>)]
    ReverseBidsSortedByPrice {
        start_before: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Get all bids for a specific account
    #[returns(Vec<Bid>)]
    BidsForSeller {
        seller: String,
        start_after: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Get the highest bid for a name
    #[returns(Option<Bid>)]
    HighestBid { token_id: TokenId },
    /// Show all registered ask hooks
    #[returns(HooksResponse)]
    AskHooks {},
    /// Show all registered bid hooks
    #[returns(HooksResponse)]
    BidHooks {},
    /// Show all registered sale hooks
    #[returns(HooksResponse)]
    SaleHooks {},
    /// Get the config for the contract
    #[returns(SudoParams)]
    Params {},
    /// Get the renewal queue for a specific time
    // #[returns(Vec<Ask>)]
    // RenewalQueue { time: Timestamp },
    /// Get the minter and collection
    #[returns(ConfigResponse)]
    Config {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub minter: Addr,
    pub collection: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct AskRenewPriceResponse {
    pub token_id: TokenId,
    pub price: Coin,
    pub bid: Option<Bid>,
}

#[cosmwasm_schema::cw_serde]
pub struct SaleHookMsg {
    pub token_id: String,
    pub seller: String,
    pub buyer: String,
}

impl SaleHookMsg {
    pub fn new(token_id: &str, seller: String, buyer: String) -> Self {
        SaleHookMsg {
            token_id: token_id.to_string(),
            seller,
            buyer,
        }
    }

    /// serializes the message
    pub fn into_json_binary(self) -> StdResult<Binary> {
        let msg = SaleExecuteMsg::SaleHook(self);
        to_json_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[cosmwasm_schema::cw_serde]
pub enum SaleExecuteMsg {
    SaleHook(SaleHookMsg),
}

#[cosmwasm_schema::cw_serde]
pub enum HookAction {
    Create,
    Update,
    Delete,
}

#[cosmwasm_schema::cw_serde]
pub struct AskHookMsg {
    pub ask: Ask,
}

impl AskHookMsg {
    pub fn new(ask: Ask) -> Self {
        AskHookMsg { ask }
    }

    /// serializes the message
    pub fn into_json_binary(self, action: HookAction) -> StdResult<Binary> {
        let msg = match action {
            HookAction::Create => AskHookExecuteMsg::AskCreatedHook(self),
            HookAction::Update => AskHookExecuteMsg::AskUpdatedHook(self),
            HookAction::Delete => AskHookExecuteMsg::AskDeletedHook(self),
        };
        to_json_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[cosmwasm_schema::cw_serde]
pub enum AskHookExecuteMsg {
    AskCreatedHook(AskHookMsg),
    AskUpdatedHook(AskHookMsg),
    AskDeletedHook(AskHookMsg),
}

#[cosmwasm_schema::cw_serde]
pub struct BidHookMsg {
    pub bid: Bid,
}

impl BidHookMsg {
    pub fn new(bid: Bid) -> Self {
        BidHookMsg { bid }
    }

    /// serializes the message
    pub fn into_json_binary(self, action: HookAction) -> StdResult<Binary> {
        let msg = match action {
            HookAction::Create => BidExecuteMsg::BidCreatedHook(self),
            HookAction::Update => BidExecuteMsg::BidUpdatedHook(self),
            HookAction::Delete => BidExecuteMsg::BidDeletedHook(self),
        };
        to_json_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[cosmwasm_schema::cw_serde]
pub enum BidExecuteMsg {
    BidCreatedHook(BidHookMsg),
    BidUpdatedHook(BidHookMsg),
    BidDeletedHook(BidHookMsg),
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
