use solana_program::program_error::ProgramError;


use solana_program::pubkey::Pubkey;
use solana_program::program_pack::{Pack, Sealed};

use crate::instruction::AllocationType;

use arrayref::{array_ref, array_refs, array_mut_ref, mut_array_refs};


pub struct StakeInfo {
    pub amount: u64,
    pub start_time: i64,
}

pub struct VestingSchedule {
    pub beneficiary: Pubkey,
    pub total_amount: u64,
    pub released_amount: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub allocation_type: AllocationType,
}

impl Sealed for VestingSchedule {}

impl Pack for VestingSchedule {
    const LEN: usize = 32 + 8 + 8 + 8 + 8 + 1;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut offset = 0;
        dst[offset..offset+32].copy_from_slice(self.beneficiary.as_ref());
        offset += 32;
        dst[offset..offset+8].copy_from_slice(&self.total_amount.to_le_bytes());
        offset += 8;
        dst[offset..offset+8].copy_from_slice(&self.released_amount.to_le_bytes());
        offset += 8;
        dst[offset..offset+8].copy_from_slice(&self.start_time.to_le_bytes());
        offset += 8;
        dst[offset..offset+8].copy_from_slice(&self.end_time.to_le_bytes());
        offset += 8;
        dst[offset] = self.allocation_type as u8;
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut offset = 0;
        let beneficiary = Pubkey::new(&src[offset..offset+32]);
        offset += 32;
        let total_amount = u64::from_le_bytes(src[offset..offset+8].try_into().unwrap());
        offset += 8;
        let released_amount = u64::from_le_bytes(src[offset..offset+8].try_into().unwrap());
        offset += 8;
        let start_time = i64::from_le_bytes(src[offset..offset+8].try_into().unwrap());
        offset += 8;
        let end_time = i64::from_le_bytes(src[offset..offset+8].try_into().unwrap());
        offset += 8;
        let allocation_type = match src[offset] {
            0 => AllocationType::Team,
            1 => AllocationType::Investors,
            2 => AllocationType::Liquidity,
            3 => AllocationType::Development,
            4 => AllocationType::Community,
            5 => AllocationType::Strategic,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(VestingSchedule {
            beneficiary,
            total_amount,
            released_amount,
            start_time,
            end_time,
            allocation_type,
        })
    }
}

impl VestingSchedule {

    pub fn pack(&self, output: &mut [u8]) -> Result<(), ProgramError> {
        let output = array_mut_ref![output, 0, VestingSchedule::LEN];
        let (
            beneficiary,
            total_amount,
            released_amount,
            start_time,
            end_time,
            allocation_type,
        ) = mut_array_refs![output, 32, 8, 8, 8, 8, 1];

        beneficiary.copy_from_slice(self.beneficiary.as_ref());
        *total_amount = self.total_amount.to_le_bytes();
        *released_amount = self.released_amount.to_le_bytes();
        *start_time = self.start_time.to_le_bytes();
        *end_time = self.end_time.to_le_bytes();
        allocation_type[0] = self.allocation_type as u8;

        Ok(())
    }

    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, VestingSchedule::LEN];
        let (
            beneficiary,
            total_amount,
            released_amount,
            start_time,
            end_time,
            allocation_type,
        ) = array_refs![input, 32, 8, 8, 8, 8, 1];

        Ok(VestingSchedule {
            beneficiary: Pubkey::new_from_array(*beneficiary),
            total_amount: u64::from_le_bytes(*total_amount),
            released_amount: u64::from_le_bytes(*released_amount),
            start_time: i64::from_le_bytes(*start_time),
            end_time: i64::from_le_bytes(*end_time),
            allocation_type: match allocation_type[0] {
                0 => AllocationType::Team,
                1 => AllocationType::Investors,
                2 => AllocationType::Liquidity,
                3 => AllocationType::Development,
                4 => AllocationType::Community,
                5 => AllocationType::Strategic,
                _ => return Err(ProgramError::InvalidAccountData),
            },
        })
    }
}

impl StakeInfo {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, 16];
        let (amount, start_time) = array_refs![input, 8, 8];
        Ok(Self {
            amount: u64::from_le_bytes(*amount),
            start_time: i64::from_le_bytes(*start_time),
        })
    }

    pub fn pack(&self, output: &mut [u8]) -> Result<(), ProgramError> {
        let output = array_mut_ref![output, 0, 16];
        let (amount_out, start_time_out) = mut_array_refs![output, 8, 8];
        *amount_out = self.amount.to_le_bytes();
        *start_time_out = self.start_time.to_le_bytes();
        Ok(())
    }
}

