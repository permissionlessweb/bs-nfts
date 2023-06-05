use cosmwasm_std::StdError;
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

    #[error("Only one of duration or max_editions can be set")]
    DurationOrMaxEditions {},

    #[error("Seller fee bps must be less than 10000")]
    SellerFeeBps {},

    #[error("Referral fee bps must be less than 10000")]
    ReferralFeeBps {},

    #[error("Contributors must not be empty")]
    ContributorsEmpty {},

    #[error("Contributors must not be greater than 100")]
    ContributorsTooMany {},

    #[error("Bs721 already linked")]
    Bs721AlreadyLinked {},

    #[error("Unknown reply id")]
    UnknownReplyId {},

    #[error("Royalty contract already linked")]
    RoyaltyContractAlreadyLinked {},

    #[error("Drop not started yet")]
    SaleNotStarted {},

    #[error("Bs721 not linked")]
    Bs721NotLinked {},

    #[error("Not enough tokens")]
    NotEnoughTokens {},

    #[error("Not enough funds")]
    NotEnoughFunds {},
}
