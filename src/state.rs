use solana_program::{
    program_error::ProgramError,
    program_option::COption,
    pubkey::Pubkey,
    program_pack::{IsInitialized, Pack, Sealed},
};
use num_enum::TryFromPrimitive;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mint {
    pub mint_authority: COption<Pubkey>,
    pub supply: u64,
    pub decimals: u8,
    pub is_initialized: bool,
}

impl Sealed for Mint {}

impl IsInitialized for Mint {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Mint {
    const LEN: usize = 46;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, 46];

        let (mint_authority, supply, decimals, is_initialized) =
            array_refs![src, 36, 8, 1, 1];

        let mint_authority = unpack_coption_key(mint_authority)?;
        let supply = u64::from_le_bytes(*supply);
        let decimals = decimals[0];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Mint {
            mint_authority,
            supply,
            decimals,
            is_initialized,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 46];

        let (
            mint_authority_dst,
            supply_dst,
            decimals_dst,
            is_initialized_dst,
        ) = mut_array_refs![dst, 36, 8, 1, 1];

        let &Mint {
            ref mint_authority,
            supply,
            decimals,
            is_initialized,
        } = self;

        pack_coption_key(mint_authority, mint_authority_dst);
        *supply_dst = supply.to_le_bytes();
        decimals_dst[0] = decimals;
        is_initialized_dst[0] = is_initialized as u8;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Account {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub delegate: COption<Pubkey>,
    pub delegated_amount: u64,
    pub state: AccountState,
}

impl Sealed for Account {}

impl IsInitialized for Account {
    fn is_initialized(&self) -> bool {
        self.state != AccountState::Uninitialized
    }
}

impl Pack for Account {
    const LEN: usize = 117;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, 117];
        
        let (mint, owner, amount, delegate, delegated_amount, state) =
            array_refs![src, 32, 32, 8, 36, 8, 1];

        Ok(Account {
            mint: Pubkey::new_from_array(*mint),
            owner: Pubkey::new_from_array(*owner),
            amount: u64::from_le_bytes(*amount),
            delegate: unpack_coption_key(delegate)?,
            delegated_amount: u64::from_le_bytes(*delegated_amount),
            state: AccountState::try_from_primitive(state[0])
                .or(Err(ProgramError::InvalidAccountData))?,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 117];
        let (
            mint_dst,
            owner_dst,
            amount_dst,
            delegate_dst,
            delegated_amount_dst,
            state_dst,
        ) = mut_array_refs![dst, 32, 32, 8, 36, 8, 1];

        let &Account {
            ref mint,
            ref owner,
            amount,
            ref delegate,
            delegated_amount,
            state,
        } = self;

        mint_dst.copy_from_slice(mint.as_ref());
        owner_dst.copy_from_slice(owner.as_ref());
        *amount_dst = amount.to_le_bytes();
        pack_coption_key(delegate, delegate_dst);
        state_dst[0] = state as u8;
        *delegated_amount_dst = delegated_amount.to_le_bytes();
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, TryFromPrimitive)]
pub enum AccountState {
    Uninitialized,
    Initialized,
}

impl Default for AccountState {
    fn default() -> Self {
        AccountState::Uninitialized
    }
}

fn pack_coption_key(src: &COption<Pubkey>, dst: &mut [u8; 36]) {
    let (tag, body) = mut_array_refs![dst, 4, 32];
    match src {
        COption::Some(key) => {
            *tag = [1, 0, 0, 0];
            body.copy_from_slice(key.as_ref());
        }
        COption::None => {
            *tag = [0; 4];
        }
    }
}

fn unpack_coption_key(src: &[u8; 36]) -> Result<COption<Pubkey>, ProgramError> {
    let (tag, body) = array_refs![src, 4, 32];
    match *tag {
        [0, 0, 0, 0] => Ok(COption::None),
        [1, 0, 0, 0] => Ok(COption::Some(Pubkey::new_from_array(*body))),
        _ => Err(ProgramError::InvalidAccountData),
    }
}
