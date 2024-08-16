use bs_controllers::HookError;
use cosmwasm_std::{Coin, Instantiate2AddressError, StdError, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    BidPaymentError(#[from] PaymentError),

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error("{0}")]
    Hook(#[from] HookError),

    #[error("AlreadySetup")]
    AlreadySetup {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("NotApproved")]
    NotApproved {},

    #[error("UnauthorizedMinter")]
    UnauthorizedMinter {},

    #[error("InsufficientRenewalFunds: expected {expected}, actual {actual}")]
    InsufficientRenewalFunds { expected: Coin, actual: Coin },

    #[error("UnauthorizedOwner")]
    UnauthorizedOwner {},

    #[error("UnauthorizedOperator")]
    UnauthorizedOperator {},

    #[error("InvalidPrice")]
    InvalidPrice {},

    #[error("InvalidDuration")]
    InvalidDuration {},

    #[error("NoRenewalFund")]
    NoRenewalFund {},

    #[error("AskUnchanged")]
    AskUnchanged {},

    #[error("AskNotFound")]
    AskNotFound {},

    #[error("CannotProcessFutureRenewal")]
    CannotProcessFutureRenewal {},

    #[error("Cannot remove ask with existing bids")]
    ExistingBids {},

    #[error("PriceTooSmall: {0}")]
    PriceTooSmall(Uint128),

    #[error("InvalidListingFee: {0}")]
    InvalidListingFee(Uint128),

    #[error("Invalid finders fee bps: {0}")]
    InvalidTradingFeeBps(u64),

    #[error("Contract got an unexpected Reply")]
    UnexpectedReply(),
}
