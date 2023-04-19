use cosmwasm_std::{Coin, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Price cannot be zero")]
    ZeroPrice {},

    #[error("Max editions cannot be zero")]
    ZeroEditions {},

    #[error("BS721 contract already linked")]
    Bs721AlreadyLinked {},

    #[error("Royalty contract already linked")]
    RoyaltyAlreadyLinked {},

    #[error("BS721 contract not linked")]
    Bs721NotLinked {},

    #[error("Unknown reply id")]
    UnknownReplyId {},

    #[error("Contract is sold out")]
    SoldOut {},

    #[error("Coins sent are invalid")]
    InvalidFunds {},

    #[error("Invalid payment amount {0} != {1}")]
    InvalidPaymentAmount(Coin, Coin),

    #[error("Launchpad not started")]
    NotStarted {},
}
