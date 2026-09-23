use borsh::{ BorshDeserialize, BorshSerialize };
use solana_program::{
    account_info::{ AccountInfo, next_account_info },
    entrypoint::{ ProgramResult },
    entrypoint,
    msg,
    program::invoke_signed,
    program_error::ProgramError::InvalidSeeds,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use solana_program::program_error::ProgramError;
use solana_system_interface::{ self, instruction };

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    msg!("Program: {}", program_id);
    msg!("Accounts supplied: {}", accounts.len());
    msg!("Instruction bytes {:?}", instruction_data);

    match instruction_data {
        [0] => {
            msg!("Initialize requested");
            initialize(program_id, accounts)
        }
        [1] => {
            msg!("Increment requested");
            increment(program_id, accounts)
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let mut account_iter = accounts.iter();
    let counter_info = next_account_info(&mut account_iter)?;
    let authority_info = next_account_info(&mut account_iter)?;
    let system_program = next_account_info(&mut account_iter)?;

    if system_program.key != &solana_system_interface::program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !authority_info.is_writable {
        return Err(ProgramError::InvalidArgument);
    }

    if !counter_info.is_writable {
        return Err(ProgramError::InvalidArgument);
    }

    if counter_info.owner != system_program.key {
        return Err(ProgramError::IllegalOwner);
    }

    let (expected_pda, bump) = Pubkey::find_program_address(&[b"vault"], program_id);

    if *counter_info.key != expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    if !counter_info.data_is_empty() || **counter_info.try_borrow_lamports()? > 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let counter_rent = Rent::get()?.minimum_balance(Counter::LEN);
    let create_ix = instruction::create_account(
        authority_info.key,
        counter_info.key,
        counter_rent,
        Counter::LEN as u64,
        program_id
    );

    let signer_seeds: &[&[u8]] = &[b"vault", &[bump]];

    invoke_signed(
        &create_ix,
        &[authority_info.clone(), counter_info.clone(), system_program.clone()],
        &[signer_seeds]
    )?;

    let count = Counter {
        authority: *authority_info.key,
        count: 0,
    };

    let mut data = counter_info.try_borrow_mut_data()?;
    let mut output = &mut data[..];

    count.serialize(&mut output).map_err(|_| ProgramError::InvalidAccountData)?;

    Ok(())
}

fn increment(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let mut account_iter = accounts.iter();
    let counter_info = next_account_info(&mut account_iter)?;
    let authority_info = next_account_info(&mut account_iter)?;

    if counter_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !counter_info.is_writable {
        return Err(ProgramError::InvalidArgument);
    }

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (expected_pda, _) = Pubkey::find_program_address(&[b"vault"], program_id);

    if counter_info.key != &expected_pda {
        return Err(InvalidSeeds);
    }

    let mut data = counter_info.try_borrow_mut_data()?;
    let mut counter = Counter::try_from_slice(&data).map_err(|_| ProgramError::InvalidAccountData)?;
    if authority_info.key != &counter.authority {
        return Err(ProgramError::IncorrectAuthority);
    }

    counter.count = counter.count.checked_add(1).ok_or(ProgramError::ArithmeticOverflow)?;

    let mut output = &mut data[..];
    counter.serialize(&mut output).map_err(|_| ProgramError::InvalidAccountData)
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Counter {
    pub authority: Pubkey,
    pub count: u64,
}

impl Counter {
    pub const LEN: usize = 32 + 8;
}
