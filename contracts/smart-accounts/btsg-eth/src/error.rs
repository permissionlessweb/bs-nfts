use cosmwasm_std::{StdError, VerificationError};

use cw_ownable::OwnershipError;
use saa::AuthError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    VerificationError(#[from] VerificationError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),
    
    #[error("{0}")]
    AuthError(#[from] AuthError),

    #[error("unauthorized")]
    Unauthorized {},
}
