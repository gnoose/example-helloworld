use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program::invoke_signed,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};
use std::convert::TryInto;


/// Amount of bytes of account data to allocate
pub const SIZE: usize = 42;

/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GreetingAccount {
    /// number of greetings
    pub counter: u32,
}

pub enum SolanaInstruction {
    ExampleInstruction {
        amount: u64,
    },
    CPIInstruction,
    TransferInstruction {
        amount: u64,
    },
}

impl SolanaInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            0 => Self::ExampleInstruction {
                amount: Self::unpack_amount(rest)?,
            },
            1 => Self::CPIInstruction,
            2 => Self::TransferInstruction {
                amount: Self::unpack_amount(rest)?,
            },
            _ => return Err(ProgramError::InvalidInstructionData.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(amount)
    }
}

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    msg!("Hello World Rust program entrypoint");
    let instruction = SolanaInstruction::unpack(instruction_data)?;

    match instruction {
        SolanaInstruction::ExampleInstruction { amount } => {
            msg!("Instruction: ExampleInstruction");
            process_example(accounts, amount, program_id)
        },
        SolanaInstruction::CPIInstruction => {
            msg!("Instruction: CPIInstruction");
            process_cpi(accounts, program_id)
        },
        SolanaInstruction::TransferInstruction { amount } => {
            msg!("Instruction: TransferInstruction");
            process_transfer(accounts, amount, program_id)
        }
    }
}

pub fn process_transfer(
    accounts: &[AccountInfo],
    amount: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    Ok(())
}

pub fn process_example(
    accounts: &[AccountInfo],
    amount: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    // Get the account to say hello to
    let account = next_account_info(accounts_iter)?;

    // Invoke the system program to allocate account data
    let (_authority_pubkey, nonce) =
        Pubkey::find_program_address(&[program_id.as_ref()], &program_id);

    // The account must be owned by the program in order to modify its data
    if account.owner != program_id {
        msg!("Greeted account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Increment and store the number of times the account has been greeted
    let mut greeting_account = GreetingAccount::try_from_slice(&account.data.borrow())?;
    greeting_account.counter += 1;
    greeting_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    msg!("Greeted {} time(s)!", greeting_account.counter);

    Ok(())
}

pub fn process_cpi(
    accounts: &[AccountInfo], // The account to say hello to
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
) -> ProgramResult {
    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    // Get the account to say hello to
    let account = next_account_info(accounts_iter)?;

    // Get the account to say hello to
    let allocated_info = next_account_info(accounts_iter)?;


    // Invoke the system program to allocate account data
    let (_authority_pubkey, nonce) =
        Pubkey::find_program_address(&[program_id.as_ref()], &program_id);

    let swap_bytes = program_id.to_bytes();
    let authority_signature_seeds = [&swap_bytes[..32], &[nonce]];
    let signers = &[&authority_signature_seeds[..]];
    invoke_signed(
        &system_instruction::allocate(allocated_info.key, SIZE as u64),
        // Order doesn't matter and this slice could include all the accounts and be:
        // `&accounts`
        &[
            account.clone(), // program being invoked also needs to be included
            allocated_info.clone(),
        ],
        signers,
    )?;

    // The account must be owned by the program in order to modify its data
    if account.owner != program_id {
        msg!("Greeted account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Increment and store the number of times the account has been greeted
    let mut greeting_account = GreetingAccount::try_from_slice(&account.data.borrow())?;
    greeting_account.counter += 1;
    greeting_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    msg!("Greeted {} time(s)!", greeting_account.counter);

    Ok(())
}

// Sanity tests
#[cfg(test)]
mod test {
    use super::*;
    use solana_program::clock::Epoch;
    use std::mem;

    #[test]
    fn test_sanity() {
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let mut lamports = 0;
        let mut data = vec![0; mem::size_of::<u32>()];
        let owner = Pubkey::default();
        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            Epoch::default(),
        );
        let instruction_data: Vec<u8> = Vec::new();

        let accounts = vec![account];

        assert_eq!(
            GreetingAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            0
        );
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        assert_eq!(
            GreetingAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            1
        );
        process_instruction(&program_id, &accounts, &instruction_data).unwrap();
        assert_eq!(
            GreetingAccount::try_from_slice(&accounts[0].data.borrow())
                .unwrap()
                .counter,
            2
        );
    }
}
