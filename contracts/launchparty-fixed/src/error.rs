use cosmwasm_std::{Coin, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("unauthorized")]
    Unauthorized {},

    #[error("max editions cannot be zero")]
    ZeroEditions {},

    #[error("party duration cannot be zero")]
    ZeroDuration {},

    #[error("BS721 contract already linked")]
    Bs721AlreadyLinked {},

    #[error("royalties contract already linked")]
    RoyaltiesAlreadyLinked {},

    #[error("BS721 contract not linked")]
    Bs721NotLinked {},

    #[error("unknown reply id")]
    UnknownReplyId {},

    #[error("contract is sold out")]
    SoldOut {},

    #[error("coins sent are invalid")]
    InvalidFunds {},

    #[error("invalid payment amount {0} != {1}")]
    InvalidPaymentAmount(Coin, Coin),

    #[error("launchpad not started")]
    NotStarted {},

    #[error("party has ended")]
    PartyEnded {},

    #[error("{profile} fee bps must be less than 10000")]
    FeeBps { profile: String},
}
