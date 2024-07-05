use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    program_pack::Pack,
    sysvar::clock::Clock,
    program::invoke,
    rent::Rent,
    sysvar::Sysvar,
    msg,
};

use solana_program::program_pack::IsInitialized;
use solana_program::account_info::next_account_info;
use spl_token::state::{Mint, Account as TokenAccount};
use crate::state::{StakeInfo, VestingSchedule};
use crate::instruction::{TokenInstruction, AllocationType};
use crate::error::TokenError;
use crate::token_info::TokenInfo;
use solana_program::program_option::COption;
use solana_program::bpf_loader_upgradeable;

use std::convert::TryFrom;

pub struct Processor;

impl IsInitialized for TokenInfo {
    fn is_initialized(&self) -> bool {
        self.total_supply != 0
    }
}

impl Processor {

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = TokenInstruction::unpack(instruction_data)?;

        match instruction {
            TokenInstruction::InitializeMint { decimals } => {
                solana_program::msg!("Instruction: Initialize Mint");
                Self::process_initialize_mint(accounts, decimals, program_id)
            }
            TokenInstruction::InitializeAccount => {
                solana_program::msg!("Instruction: Initialize Account");
                Self::process_initialize_account(accounts, program_id)
            }
            TokenInstruction::Transfer { amount } => {
                solana_program::msg!("Instruction: Transfer");
                Self::process_transfer(accounts, amount, program_id)
            }
            TokenInstruction::Burn { amount } => {
                solana_program::msg!("Instruction: Burn");
                Self::process_burn(accounts, amount, program_id)
            }
            TokenInstruction::MintTo { amount } => {
                solana_program::msg!("Instruction: Mint To");
                Self::process_mint_to(accounts, amount, program_id)
            }
            TokenInstruction::Freeze => {
                solana_program::msg!("Instruction: Freeze");
                Self::process_freeze(accounts, program_id)
            }
            TokenInstruction::Thaw => {
                solana_program::msg!("Instruction: Thaw");
                Self::process_thaw(accounts, program_id)
            }
            TokenInstruction::SetAuthority { authority_type, new_authority } => {
                solana_program::msg!("Instruction: Set Authority");
                Self::process_set_authority(accounts, authority_type, new_authority, program_id)
            }
            TokenInstruction::Stake { amount } => {
                solana_program::msg!("Instruction: Stake");
                Self::process_stake(accounts, amount, program_id)
            }
            TokenInstruction::Unstake { amount } => {
                solana_program::msg!("Instruction: Unstake");
                Self::process_unstake(accounts, amount, program_id)
            }
            TokenInstruction::UpgradeProgram => {
                solana_program::msg!("Instruction: Upgrade Program");
                Self::process_upgrade_program(accounts, program_id)
            }
            TokenInstruction::InitializeTokenInfo => {
                solana_program::msg!("Instruction: Initialize Token Info");
                Self::process_initialize_token_info(accounts, program_id)
            },
            TokenInstruction::CreateVestingSchedule { beneficiary, allocation_type, amount, start_time, end_time } => {
                solana_program::msg!("Instruction: Create Vesting Schedule");
                Self::process_create_vesting_schedule(accounts, beneficiary, allocation_type, amount, start_time, end_time, program_id)
            },
            TokenInstruction::ReleaseVestedTokens => {
                solana_program::msg!("Instruction: Release Vested Tokens");
                Self::process_release_vested_tokens(accounts, program_id)
            },
            
        }
    }

    fn process_initialize_mint(
        accounts: &[AccountInfo],
        decimals: u8,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_account = next_account_info(account_info_iter)?;
        let mint_authority = next_account_info(account_info_iter)?;
        let freeze_authority = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    
        if !rent.is_exempt(mint_account.lamports(), mint_account.data_len()) {
            return Err(TokenError::NotRentExempt.into());
        }
    
        if mint_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId.into());
        }
    
        let freeze_authority_option = if freeze_authority.key != &Pubkey::default() {
            Some(freeze_authority.key)
        } else {
            None
        };
                
    
        invoke(
            &spl_token::instruction::initialize_mint(
                &spl_token::id(),
                mint_account.key,
                mint_authority.key,
                freeze_authority_option,
                decimals,
            )?,
            &[mint_account.clone(), rent.to_account_info()],

        )
    }

    fn process_initialize_account(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let account = next_account_info(account_info_iter)?;
        let mint = next_account_info(account_info_iter)?;
        let owner = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    
        if !rent.is_exempt(account.lamports(), account.data_len()) {
            return Err(TokenError::NotRentExempt.into());
        }
    
        if account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId.into());
        }
    
        invoke(
            &spl_token::instruction::initialize_account(
                &spl_token::id(),
                account.key,
                mint.key,
                owner.key,
            )?,
            &[account.clone(), mint.clone(), owner.clone(), rent.to_account_info()],
        )
    }

    fn process_transfer(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let source = next_account_info(account_info_iter)?;
        let destination = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
    
        if source.owner != program_id || destination.owner != program_id {
            return Err(ProgramError::IncorrectProgramId.into());
        }
    
        let source_account = TokenAccount::unpack(&source.data.borrow())?;
        if source_account.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }
    
        if source_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }
    
        invoke(
            &spl_token::instruction::transfer(
                &spl_token::id(),
                source.key,
                destination.key,
                authority.key,
                &[],
                amount,
            )?,
            &[source.clone(), destination.clone(), authority.clone()],
        )
    }

    fn process_burn(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let account = next_account_info(account_info_iter)?;
        let mint = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
    
        if account.owner != program_id || mint.owner != program_id {
            return Err(ProgramError::IncorrectProgramId.into());
        }
    
        let token_account = TokenAccount::unpack(&account.data.borrow())?;
        if token_account.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }
    
        if token_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }
    
        invoke(
            &spl_token::instruction::burn(
                &spl_token::id(),
                account.key,
                mint.key,
                authority.key,
                &[],
                amount,
            )?,
            &[account.clone(), mint.clone(), authority.clone()],
        )
    }

    fn process_mint_to(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint = next_account_info(account_info_iter)?;
        let account = next_account_info(account_info_iter)?;
        let owner = next_account_info(account_info_iter)?;
    
        if mint.owner != program_id || account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId.into());
        }
    
        let mint_data = Mint::unpack(&mint.data.borrow())?;
        if mint_data.mint_authority.is_none() || mint_data.mint_authority.unwrap() != *owner.key {
            return Err(TokenError::InvalidAuthority.into());
        }
    
        invoke(
            &spl_token::instruction::mint_to(
                &spl_token::id(),
                mint.key,
                account.key,
                owner.key,
                &[],
                amount,
            )?,
            &[mint.clone(), account.clone(), owner.clone()],
        )
    }

    fn process_freeze(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let account = next_account_info(account_info_iter)?;
        let mint = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
    
        if account.owner != program_id || mint.owner != program_id {
            return Err(ProgramError::IncorrectProgramId.into());
        }
    
        let mint_data = Mint::unpack(&mint.data.borrow())?;
        if mint_data.freeze_authority.is_none() || mint_data.freeze_authority.unwrap() != *authority.key {
            return Err(TokenError::InvalidAuthority.into());
        }
    
        invoke(
            &spl_token::instruction::freeze_account(
                &spl_token::id(),
                account.key,
                mint.key,
                authority.key,
                &[],
            )?,
            &[account.clone(), mint.clone(), authority.clone()],
        )
    }

    fn process_thaw(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let account = next_account_info(account_info_iter)?;
        let mint = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
    
        if account.owner != program_id || mint.owner != program_id {
            return Err(ProgramError::IncorrectProgramId.into());
        }
    
        let mint_data = Mint::unpack(&mint.data.borrow())?;
        if mint_data.freeze_authority.is_none() || mint_data.freeze_authority.unwrap() != *authority.key {
            return Err(TokenError::InvalidAuthority.into());
        }
    
        invoke(
            &spl_token::instruction::thaw_account(
                &spl_token::id(),
                account.key,
                mint.key,
                authority.key,
                &[],
            )?,
            &[account.clone(), mint.clone(), authority.clone()],
        )
    }

    fn process_set_authority(
        accounts: &[AccountInfo],
        authority_type: u8,
        new_authority: Option<Pubkey>,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let account_or_mint = next_account_info(account_info_iter)?;
        let current_authority = next_account_info(account_info_iter)?;
    
        if account_or_mint.owner != program_id {
            return Err(ProgramError::IncorrectProgramId.into());
        }

        let authority_type = spl_token::instruction::AuthorityType::try_from(authority_type)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    
        invoke(
            &spl_token::instruction::set_authority(
                &spl_token::id(),
                account_or_mint.key,
                new_authority.as_ref(),
                spl_token::instruction::AuthorityType::from(authority_type)?,
                current_authority.key,
                &[],
            )?,
            &[account_or_mint.clone(), current_authority.clone()],
        )
    }

    fn process_stake(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let stake_account = next_account_info(account_info_iter)?;
        let token_account = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
        let clock = Clock::from_account_info(next_account_info(account_info_iter)?)?;
    
        if stake_account.owner != program_id || token_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId.into());
        }
    
        let mut stake_info = StakeInfo::unpack(&stake_account.data.borrow())?;
        let mut token_account_info = TokenAccount::unpack(&token_account.data.borrow())?;
    
        if token_account_info.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }
    
        token_account_info.amount -= amount;
        stake_info.amount += amount;
        stake_info.start_time = clock.unix_timestamp;
    
        StakeInfo::pack(&stake_info, &mut stake_account.data.borrow_mut())?;
        TokenAccount::pack(token_account_info, &mut token_account.data.borrow_mut())?;

        Ok(())
}


fn process_unstake(
    accounts: &[AccountInfo],
    amount: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let stake_account = next_account_info(account_info_iter)?;
    let token_account = next_account_info(account_info_iter)?;
    let authority = next_account_info(account_info_iter)?;
    let clock = Clock::from_account_info(next_account_info(account_info_iter)?)?;

    if stake_account.owner != program_id || token_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    let mut stake_info = StakeInfo::unpack(&stake_account.data.borrow())?;
    let mut token_account_info = TokenAccount::unpack(&token_account.data.borrow())?;

    if stake_info.amount < amount {
        return Err(TokenError::InsufficientFunds.into());
    }

    // 7 days stacking
    if clock.unix_timestamp - stake_info.start_time < 7 * 24 * 60 * 60 {
        return Err(ProgramError::Custom(1)); 
    }

    stake_info.amount -= amount;
    token_account_info.amount += amount;

    StakeInfo::pack(&stake_info, &mut stake_account.data.borrow_mut())?;
    TokenAccount::pack(token_account_info, &mut token_account.data.borrow_mut())?;

    Ok(())
}

fn process_upgrade_program(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let program_account = next_account_info(account_info_iter)?;
    let buffer_account = next_account_info(account_info_iter)?;
    let spill_account = next_account_info(account_info_iter)?;
    let authority = next_account_info(account_info_iter)?;
    let rent_sysvar = next_account_info(account_info_iter)?;

    if program_account.key != program_id {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    invoke(
        &bpf_loader_upgradeable::upgrade(
            program_account.key,
            buffer_account.key,
            spill_account.key,
            authority.key,
        ),
        &[
            program_account.clone(),
            buffer_account.clone(),
            spill_account.clone(),
            authority.clone(),
            rent_sysvar.clone(),
        ],
    )
}

fn process_initialize_token_info(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let token_info_account = next_account_info(account_info_iter)?;
    let mint_authority = next_account_info(account_info_iter)?;
    let mint = next_account_info(account_info_iter)?;

    if token_info_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    if !token_info_account.data.borrow().iter().all(|&x| x == 0) {
        return Err(TokenError::AlreadyInitialized.into());
    }

    let total_supply: u64 = 50_000_000_000 * 10u64.pow(9); 

    let token_info = TokenInfo {
        total_supply,
        team_allocation: total_supply / 10, 
        investors_allocation: total_supply / 5, 
        liquidity_reserve: total_supply * 15 / 100, 
        development_reserve: total_supply / 5, 
        community_rewards: total_supply / 4, 
        strategic_reserve: total_supply / 10, 
        mint_authority: *mint_authority.key,
        mint: *mint.key,
    };

    TokenInfo::pack(token_info, &mut token_info_account.data.borrow_mut())?;

    
    let mint_data = Mint {
        mint_authority: COption::Some(*mint_authority.key),
        supply: total_supply,
        decimals: 9,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    Mint::pack(mint_data, &mut mint.data.borrow_mut())?;

    Ok(())
}

fn process_create_vesting_schedule(
    accounts: &[AccountInfo],
    beneficiary: Pubkey,
    allocation_type: AllocationType,
    amount: u64,
    start_time: i64,
    end_time: i64,
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let vesting_account = next_account_info(account_info_iter)?;
    let token_info_account = next_account_info(account_info_iter)?;
    let authority = next_account_info(account_info_iter)?;

    if vesting_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    let mut token_info = TokenInfo::unpack(&token_info_account.data.borrow())?;
    if *authority.key != token_info.mint_authority {
        return Err(TokenError::InvalidAuthority.into());
    }

    let allocation = match allocation_type {
        AllocationType::Team => &mut token_info.team_allocation,
        AllocationType::Investors => &mut token_info.investors_allocation,
        AllocationType::Liquidity => &mut token_info.liquidity_reserve,
        AllocationType::Development => &mut token_info.development_reserve,
        AllocationType::Community => &mut token_info.community_rewards,
        AllocationType::Strategic => &mut token_info.strategic_reserve,
    };

    if amount > *allocation {
        return Err(TokenError::InsufficientFunds.into());
    }

    let vesting_schedule = VestingSchedule {
        beneficiary,
        total_amount: amount,
        released_amount: 0,
        start_time,
        end_time,
        allocation_type,
    };

    VestingSchedule::pack(&vesting_schedule, &mut vesting_account.data.borrow_mut())?;

    
    *allocation -= amount;
    TokenInfo::pack(token_info, &mut token_info_account.data.borrow_mut())?;

    Ok(())
}

fn process_release_vested_tokens(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let vesting_account = next_account_info(account_info_iter)?;
    let token_account = next_account_info(account_info_iter)?;
    let mint = next_account_info(account_info_iter)?;
    let authority = next_account_info(account_info_iter)?;
    let clock = Clock::from_account_info(next_account_info(account_info_iter)?)?;

    if vesting_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId.into());
    }

    let mut vesting_schedule = VestingSchedule::unpack(&vesting_account.data.borrow())?;

    if clock.unix_timestamp < vesting_schedule.start_time {
        return Err(TokenError::VestingNotStarted.into());
    }

    let total_time = vesting_schedule.end_time - vesting_schedule.start_time;
    let elapsed_time = clock.unix_timestamp - vesting_schedule.start_time;
    let vested_amount = if elapsed_time >= total_time {
        vesting_schedule.total_amount
    } else {
        vesting_schedule.total_amount * elapsed_time as u64 / total_time as u64
    };

    let releasable_amount = vested_amount.saturating_sub(vesting_schedule.released_amount);

    if releasable_amount == 0 {
        return Err(TokenError::NoTokensToRelease.into());
    }

  
    let transfer_instruction = spl_token::instruction::transfer(
        &spl_token::id(),
        mint.key,
        token_account.key,
        &vesting_schedule.beneficiary,
        &[&authority.key],
        releasable_amount,
    )?;

    invoke(
        &transfer_instruction,
        &[mint.clone(), token_account.clone(), authority.clone()],
    )?;

    vesting_schedule.released_amount += releasable_amount;
    VestingSchedule::pack(&vesting_schedule, &mut vesting_account.data.borrow_mut())?;

    Ok(())
}
}
