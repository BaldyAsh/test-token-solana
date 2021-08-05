use solana_program::{
    pubkey::Pubkey,
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_option::COption,
    msg,
    program_pack::{IsInitialized, Pack},
    sysvar::{rent::Rent, Sysvar},
};
use crate::{
    error::TokenError,
    instruction::{TokenInstruction},
    state::{Account, AccountState, Mint},
};


pub struct Processor {}
impl Processor {
    pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = TokenInstruction::unpack(input)?;

        match instruction {
            TokenInstruction::InitializeMint {
                decimals,
                mint_authority,
            } => {
                msg!("Instruction: InitializeMint");
                Self::process_initialize_mint(accounts, decimals, mint_authority)
            }
            TokenInstruction::InitializeAccount => {
                msg!("Instruction: InitializeAccount");
                Self::process_initialize_account(accounts)
            }
            TokenInstruction::Transfer { amount } => {
                msg!("Instruction: Transfer");
                Self::process_transfer(accounts, amount)
            }
            TokenInstruction::Approve { amount } => {
                msg!("Instruction: Approve");
                Self::process_approve(accounts, amount)
            }
            TokenInstruction::MintTo { amount } => {
                msg!("Instruction: MintTo");
                Self::process_mint_to(accounts, amount)
            }
            TokenInstruction::Burn { amount } => {
                msg!("Instruction: Burn");
                Self::process_burn(accounts, amount)
            }
        }
    }

    fn process_initialize_mint(
        accounts: &[AccountInfo],
        decimals: u8,
        mint_authority: Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        let mint_data_len = mint_info.data_len();
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        let mut mint = Mint::unpack_unchecked(&mint_info.data.borrow())?;
        if mint.is_initialized {
            return Err(TokenError::AlreadyInUse.into());
        }

        if !rent.is_exempt(mint_info.lamports(), mint_data_len) {
            return Err(TokenError::NotRentExempt.into());
        }

        mint.mint_authority = COption::Some(mint_authority);
        mint.decimals = decimals;
        mint.is_initialized = true;

        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_initialize_account(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let new_account_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let owner = next_account_info(account_info_iter)?.key;
        let new_account_info_data_len = new_account_info.data_len();
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        let mut account = Account::unpack_unchecked(&new_account_info.data.borrow())?;
        if account.is_initialized() {
            return Err(TokenError::AlreadyInUse.into());
        }

        if !rent.is_exempt(new_account_info.lamports(), new_account_info_data_len) {
            return Err(TokenError::NotRentExempt.into());
        }

        let _ = Mint::unpack(&mint_info.data.borrow_mut())
                .map_err(|_| Into::<ProgramError>::into(TokenError::InvalidMint))?;

        account.mint = *mint_info.key;
        account.owner = *owner;
        account.delegate = COption::None;
        account.delegated_amount = 0;
        account.state = AccountState::Initialized;
        account.amount = 0;

        Account::pack(account, &mut new_account_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_transfer(
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let source_account_info = next_account_info(account_info_iter)?;
        let dest_account_info = next_account_info(account_info_iter)?;
        if source_account_info.key == dest_account_info.key {
            return Err(TokenError::SelfTransfer.into());
        }

        let authority_info = next_account_info(account_info_iter)?;

        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        let mut dest_account = Account::unpack(&dest_account_info.data.borrow())?;

        if source_account.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }
        if source_account.mint != dest_account.mint {
            return Err(TokenError::MintMismatch.into());
        }

        match source_account.delegate {
            COption::Some(ref delegate) if authority_info.key == delegate => {
                Self::validate_owner(
                    delegate,
                    authority_info,
                )?;

                if source_account.delegated_amount < amount {
                    return Err(TokenError::InsufficientFunds.into());
                }
                
                // Remove delegated amount from transfer authority
                source_account.delegated_amount = source_account
                    .delegated_amount
                    .checked_sub(amount)
                    .ok_or(TokenError::Overflow)?;

                if source_account.delegated_amount == 0 {
                    source_account.delegate = COption::None;
                }
            }
            _ => Self::validate_owner(
                &source_account.owner,
                authority_info,
            )?,
        };

        source_account.amount = source_account
            .amount
            .checked_sub(amount)
            .ok_or(TokenError::Overflow)?;
        dest_account.amount = dest_account
            .amount
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;

        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;
        Account::pack(dest_account, &mut dest_account_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_approve(
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let source_account_info = next_account_info(account_info_iter)?;
        let delegate_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;

        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;

        Self::validate_owner(
            &source_account.owner,
            owner_info,
        )?;

        source_account.delegate = COption::Some(*delegate_info.key);
        source_account.delegated_amount = amount;

        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_mint_to(
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        let dest_account_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;

        let mut dest_account = Account::unpack(&dest_account_info.data.borrow())?;
        if mint_info.key != &dest_account.mint {
            return Err(TokenError::MintMismatch.into());
        }

        let mut mint = Mint::unpack(&mint_info.data.borrow())?;
        match mint.mint_authority {
            COption::Some(mint_authority) => Self::validate_owner(
                &mint_authority,
                owner_info,
            )?,
            COption::None => return Err(TokenError::FixedSupply.into()),
        }

        dest_account.amount = dest_account
            .amount
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;

        mint.supply = mint
            .supply
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;

        Account::pack(dest_account, &mut dest_account_info.data.borrow_mut())?;
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_burn(
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let source_account_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;

        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        if source_account.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }
        if mint_info.key != &source_account.mint {
            return Err(TokenError::MintMismatch.into());
        }

        match source_account.delegate {
            COption::Some(ref delegate) if authority_info.key == delegate => {
                Self::validate_owner(
                    delegate,
                    authority_info,
                )?;

                if source_account.delegated_amount < amount {
                    return Err(TokenError::InsufficientFunds.into());
                }
                source_account.delegated_amount = source_account
                    .delegated_amount
                    .checked_sub(amount)
                    .ok_or(TokenError::Overflow)?;
                if source_account.delegated_amount == 0 {
                    source_account.delegate = COption::None;
                }
            }
            _ => Self::validate_owner(
                &source_account.owner,
                authority_info,
            )?,
        }

        source_account.amount = source_account
            .amount
            .checked_sub(amount)
            .ok_or(TokenError::Overflow)?;

        let mut mint = Mint::unpack(&mint_info.data.borrow())?;
        mint.supply = mint
            .supply
            .checked_sub(amount)
            .ok_or(TokenError::Overflow)?;

        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    fn validate_owner(
        expected_owner: &Pubkey,
        owner_account_info: &AccountInfo
    ) -> ProgramResult {
        if expected_owner != owner_account_info.key {
            return Err(TokenError::OwnerMismatch.into());
        }
        if !owner_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::*;
    use solana_program::{
        account_info::IntoAccountInfo, clock::Epoch, instruction::Instruction, sysvar::rent,
    };
    use solana_sdk::account::{
        create_account_for_test, create_is_signer_account_infos, Account as SolanaAccount,
    };

    fn do_process_instruction(
        instruction: Instruction,
        accounts: Vec<&mut SolanaAccount>,
    ) -> ProgramResult {
        let mut meta = instruction
            .accounts
            .iter()
            .zip(accounts)
            .map(|(account_meta, account)| (&account_meta.pubkey, account_meta.is_signer, account))
            .collect::<Vec<_>>();

        let account_infos = create_is_signer_account_infos(&mut meta);
        Processor::process(&instruction.program_id, &account_infos, &instruction.data)
    }

    fn do_process_instruction_dups(
        instruction: Instruction,
        account_infos: Vec<AccountInfo>,
    ) -> ProgramResult {
        Processor::process(&instruction.program_id, &account_infos, &instruction.data)
    }

    fn rent_sysvar() -> SolanaAccount {
        create_account_for_test(&Rent::default())
    }

    fn mint_minimum_balance() -> u64 {
        Rent::default().minimum_balance(Mint::get_packed_len())
    }

    fn account_minimum_balance() -> u64 {
        Rent::default().minimum_balance(Account::get_packed_len())
    }

    #[test]
    fn test_pack_unpack_mint() {
        // Mint
        let mint = Mint {
            mint_authority: COption::Some(Pubkey::new(&[1; 32])),
            supply: 42,
            decimals: 7,
            is_initialized: true,
        };
        let mut packed = vec![0; Mint::get_packed_len() + 1];
        assert_eq!(
            Err(ProgramError::InvalidAccountData),
            Mint::pack(mint, &mut packed)
        );
        let mut packed = vec![0; Mint::get_packed_len() - 1];
        assert_eq!(
            Err(ProgramError::InvalidAccountData),
            Mint::pack(mint, &mut packed)
        );
        let mut packed = vec![0; Mint::get_packed_len()];
        Mint::pack(mint, &mut packed).unwrap();
        let expect = vec![
            1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 42, 0, 0, 0, 0, 0, 0, 0, 7, 1,
        ];
        assert_eq!(packed, expect);
        let unpacked = Mint::unpack(&packed).unwrap();
        assert_eq!(unpacked, mint);
    }

    #[test]
    fn test_pack_unpack_account() {
        // Account
        let check = Account {
            mint: Pubkey::new(&[1; 32]),
            owner: Pubkey::new(&[2; 32]),
            amount: 3,
            delegate: COption::Some(Pubkey::new(&[4; 32])),
            delegated_amount: 6,
            state: AccountState::Initialized,
        };
        let mut packed = vec![0; Account::get_packed_len() + 1];
        assert_eq!(
            Err(ProgramError::InvalidAccountData),
            Account::pack(check, &mut packed)
        );
        let mut packed = vec![0; Account::get_packed_len() - 1];
        assert_eq!(
            Err(ProgramError::InvalidAccountData),
            Account::pack(check, &mut packed)
        );
        let mut packed = vec![0; Account::get_packed_len()];
        Account::pack(check, &mut packed).unwrap();
        let expect = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 3, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 6, 0, 0, 0, 0, 0, 0, 0, 1
        ];
        assert_eq!(packed, expect);
        let unpacked = Account::unpack(&packed).unwrap();
        assert_eq!(unpacked, check);
    }
}
