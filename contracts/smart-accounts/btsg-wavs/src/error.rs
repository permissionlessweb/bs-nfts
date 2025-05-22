use cosmwasm_std::{StdError, VerificationError};

use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    VerificationError(#[from] VerificationError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("Authenticator metadata must be provided when this contract to an account")]
    MissingAuthenticatorMetadata {},

    #[error("attempting to register too many WAVS operator keys. Currently hardcoded to 10")]
    TooManyWavsKeys {},

    #[error("have: {a}, want: {b}")]
    InvalidPubkeyCount { a: usize, b: usize },

    #[error("unauthorized")]
    Unauthorized {},
}
