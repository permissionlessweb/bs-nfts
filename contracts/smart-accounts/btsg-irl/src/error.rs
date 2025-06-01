
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized - only the smart account can perform this action")]
    Unauthorized {},

    #[error("Fantoken has not been created yet")]
    FantokenNotCreated {},

    #[error("Minting would exceed maximum supply")]
    ExceedsMaxSupply {},

    #[error("Unknown reply ID")]
    UnknownReplyId {},

    #[error("Invalid fantoken symbol")]
    InvalidSymbol {},

    #[error("Invalid URI format")]
    InvalidUri {},
}