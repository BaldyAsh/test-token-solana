use solana_program::{
    decode_error::DecodeError,
    program_error::{ProgramError, PrintProgramError},
    msg,
};
use thiserror::Error;
use num_traits::FromPrimitive;
use num_derive::FromPrimitive;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TokenError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Already in use")]
    AlreadyInUse,
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
    #[error("Invalid mint")]
    InvalidMint,
    #[error("Mint mismatch")]
    MintMismatch,
    #[error("Self transfer")]
    SelfTransfer,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Overflow")]
    Overflow,
    #[error("Fixed supply")]
    FixedSupply,
    #[error("Owner mismatch")]
    OwnerMismatch,
}

impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TokenError {
    fn type_of() -> &'static str {
        "TokenError"
    }
}

impl PrintProgramError for TokenError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            TokenError::NotRentExempt => msg!("Error: Lamport balance below rent-exempt threshold"),
            TokenError::InvalidInstruction => msg!("Error: Invalid instruction"),
            TokenError::AlreadyInUse => msg!("Error: Already in use"),
            TokenError::InvalidMint => msg!("Error: Invalid mint"),
            TokenError::MintMismatch => msg!("Error: Mint mismatch"),
            TokenError::SelfTransfer => msg!("Error: Self transfer"),
            TokenError::InsufficientFunds => msg!("Error: Insufficient funds"),
            TokenError::Overflow => msg!("Error: Overflow"),
            TokenError::FixedSupply => msg!("Error: Fixed supply"),
            TokenError::OwnerMismatch => msg!("Error: Owner mismatch"),
        }
    }
}
