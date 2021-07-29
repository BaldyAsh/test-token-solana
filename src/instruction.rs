use crate::{error::TokenError};
use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum TokenInstruction {
    InitializeMint {
        decimals: u8,
        mint_authority: Pubkey
    },
    InitializeAccount,
}

impl TokenInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use TokenError::InvalidInstruction;

        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => {
                let (&decimals, rest) = rest.split_first().ok_or(InvalidInstruction)?;
                let (mint_authority, _rest) = Self::unpack_pubkey(rest)?;
                Self::InitializeMint {
                    decimals,
                    mint_authority,
                }
            }
            _ => return Err(TokenError::InvalidInstruction.into()),
        })
    }

    fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
        if input.len() >= 32 {
            let (key, rest) = input.split_at(32);
            let pk = Pubkey::new(key);
            Ok((pk, rest))
        } else {
            Err(TokenError::InvalidInstruction.into())
        }
    }
}
