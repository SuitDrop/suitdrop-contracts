use cosmwasm_std::{OverflowError, StdError};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    // payment erro
    #[error("Payment error")]
    PaymentError(#[from] PaymentError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Only one ${denom:?} may be redeemed at a time")]
    InvalidRedemptionAmount { denom: String },

    // parse reply error
    #[error("Parse reply error")]
    ParseReplyError(#[from] cw_utils::ParseReplyError),

    #[error("Invalid reply id")]
    InvalidReplyId {},

    // overflow
    #[error("Overflow error")]
    OverflowError(#[from] OverflowError),
}
