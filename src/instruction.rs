use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::program_pack::Pack;
use spl_token::state::Account as TokenAccount;

use crate::error::TokenError;
use arrayref::{array_ref, array_refs};

#[derive(Debug)]
pub enum TokenInstruction {
    InitializeMint { decimals: u8 },
    InitializeAccount,
    Transfer { amount: u64 },
    Burn { amount: u64 },
    MintTo { amount: u64 },
    Freeze,
    Thaw,
    SetAuthority { authority_type: u8, new_authority: Option<Pubkey> },
    Stake { amount: u64 },
    Unstake { amount: u64 },
    UpgradeProgram,
    InitializeTokenInfo,
    CreateVestingSchedule {
        beneficiary: Pubkey,
        allocation_type: AllocationType,
        amount: u64,
        start_time: i64,
        end_time: i64,
    },
    ReleaseVestedTokens,
}

#[derive(Debug)]
pub enum AllocationType {
    Team,
    Investors,
    Liquidity,
    Development,
    Community,
    Strategic,
}

impl TokenInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use TokenError::InvalidInstruction;
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let decimals = rest.get(0).copied().ok_or(InvalidInstruction)?;
                Self::InitializeMint { decimals }
            }
            1 => Self::InitializeAccount,
            2 => {
                let amount = Self::unpack_amount(rest)?;
                Self::Transfer { amount }
            }
            3 => {
                let amount = Self::unpack_amount(rest)?;
                Self::Burn { amount }
            }
            4 => {
                let amount = Self::unpack_amount(rest)?;
                Self::MintTo { amount }
            }
            5 => Self::Freeze,
            6 => Self::Thaw,
            7 => {
                let (authority_type, rest) = rest.split_first().ok_or(InvalidInstruction)?;
                let mut new_authority = None;
                if !rest.is_empty() {
                    new_authority = Some(Pubkey::new(rest));
                }
                Self::SetAuthority {
                    authority_type: *authority_type,
                    new_authority,
                }
            }
            8 => {
                let amount = Self::unpack_amount(rest)?;
                Self::Stake { amount }
            }
            9 => {
                let amount = Self::unpack_amount(rest)?;
                Self::Unstake { amount }
            }
            10 => Self::UpgradeProgram,
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(TokenError::InvalidInstruction)?;
        Ok(amount)
    }
}
