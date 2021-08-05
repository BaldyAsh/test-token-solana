use crate::{error::TokenError};
use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::convert::TryInto;
use std::mem::size_of;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum TokenInstruction {
    InitializeMint {
        decimals: u8,
        mint_authority: Pubkey
    },
    InitializeAccount,
    Transfer { amount: u64, },
    Approve { amount: u64, },
    MintTo { amount: u64, },
    Burn { amount: u64, },
}

impl TokenInstruction {
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::InitializeMint {
                mint_authority,
                decimals,
            } => {
                buf.push(0);
                buf.push(*decimals);
                buf.extend_from_slice(mint_authority.as_ref());
            }
            Self::InitializeAccount => buf.push(1),
            Self::Transfer { amount } => {
                buf.push(2);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::Approve { amount } => {
                buf.push(3);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::MintTo { amount } => {
                buf.push(4);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::Burn { amount } => {
                buf.push(5);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
        };
        buf
    }

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
            1 => Self::InitializeAccount,
            2 | 3 | 4 | 5 => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                match tag {
                    2 => Self::Transfer { amount },
                    3 => Self::Approve { amount },
                    4 => Self::MintTo { amount },
                    5 => Self::Burn { amount },
                    _ => unreachable!(),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mint1() {
        let mint = TokenInstruction::InitializeMint {
            decimals: 2,
            mint_authority: Pubkey::new(&[1u8; 32]),
        };

        let mut packed = Vec::from([0u8, 2]);
        packed.extend_from_slice(&[1u8; 32]);

        assert_eq!(mint.pack(), packed);

        let unpacked = TokenInstruction::unpack(&packed).unwrap();
        assert_eq!(unpacked, mint);
    }

    #[test]
    fn test_mint2() {
        let mint = TokenInstruction::InitializeMint {
            decimals: 2,
            mint_authority: Pubkey::new(&[2u8; 32]),
        };

        let mut packed = Vec::from([0u8, 2]);
        packed.extend_from_slice(&[2u8; 32]);

        assert_eq!(mint.pack(), packed);

        let unpacked = TokenInstruction::unpack(&packed).unwrap();
        assert_eq!(unpacked, mint);
    }

    #[test]
    fn test_init_account1() {
        let init_account = TokenInstruction::InitializeAccount;

        let packed = Vec::from([1u8]);

        assert_eq!(init_account.pack(), packed);

        let unpacked = TokenInstruction::unpack(&packed).unwrap();

        assert_eq!(unpacked, init_account);
    }
}
