use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    program_error::{ProgramError, PrintProgramError},
    msg,
};
use thiserror::Error;
use num_traits::FromPrimitive;

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
        }
    }
}
