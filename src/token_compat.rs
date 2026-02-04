//! Token Program Compatibility Layer
//! 
//! This module provides compatibility between SPL Token (v1) and SPL Token-2022 (v2).
//! Both programs use the same instruction format for basic operations, so we can
//! use the spl_token crate with different program IDs.
//!
//! Key insight: Token-2022 uses the same instruction encoding as Token v1 for
//! basic operations (transfer, mint_to, burn, initialize_account3, initialize_mint2).

use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
};

/// SPL Token Program ID (v1)
pub const TOKEN_PROGRAM_V1: Pubkey = spl_token::id();

/// SPL Token-2022 Program ID (v2)
/// TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
pub const TOKEN_PROGRAM_V2: Pubkey = solana_program::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

/// Check if a program ID is a valid token program (v1 or v2)
#[allow(dead_code)]
pub fn is_valid_token_program(program_id: &Pubkey) -> bool {
    *program_id == TOKEN_PROGRAM_V1 || *program_id == TOKEN_PROGRAM_V2
}

/// Get the token account size (same for both v1 and v2 for basic accounts)
pub fn get_token_account_size(_token_program_id: &Pubkey) -> usize {
    // Both v1 and v2 use 165 bytes for basic token accounts
    spl_token::state::Account::LEN
}

/// Get the mint size (same for both v1 and v2 for basic mints)
#[allow(dead_code)]
pub fn get_mint_size(_token_program_id: &Pubkey) -> usize {
    // Both v1 and v2 use 82 bytes for basic mints
    spl_token::state::Mint::LEN
}

/// Create an InitializeAccount3 instruction (works for both v1 and v2)
/// InitializeAccount3 doesn't require rent sysvar
pub fn create_initialize_account3_instruction(
    token_program_id: &Pubkey,
    account: &Pubkey,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Result<solana_program::instruction::Instruction, solana_program::program_error::ProgramError> {
    // InitializeAccount3 is instruction 18 in both v1 and v2
    // Format: [instruction_type (1 byte)] + [owner pubkey (32 bytes)]
    let mut data = Vec::with_capacity(33);
    data.push(18u8); // InitializeAccount3 instruction
    data.extend_from_slice(owner.as_ref());
    
    Ok(solana_program::instruction::Instruction {
        program_id: *token_program_id,
        accounts: vec![
            solana_program::instruction::AccountMeta::new(*account, false),
            solana_program::instruction::AccountMeta::new_readonly(*mint, false),
        ],
        data,
    })
}

/// Create a Transfer instruction (works for both v1 and v2)
pub fn create_transfer_instruction(
    token_program_id: &Pubkey,
    source: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Result<solana_program::instruction::Instruction, solana_program::program_error::ProgramError> {
    // Transfer is instruction 3 in both v1 and v2
    // Format: [instruction_type (1 byte)] + [amount (8 bytes LE)]
    let mut data = Vec::with_capacity(9);
    data.push(3u8); // Transfer instruction
    data.extend_from_slice(&amount.to_le_bytes());
    
    Ok(solana_program::instruction::Instruction {
        program_id: *token_program_id,
        accounts: vec![
            solana_program::instruction::AccountMeta::new(*source, false),
            solana_program::instruction::AccountMeta::new(*destination, false),
            solana_program::instruction::AccountMeta::new_readonly(*authority, true),
        ],
        data,
    })
}

/// Create a MintTo instruction (works for both v1 and v2)
pub fn create_mint_to_instruction(
    token_program_id: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Result<solana_program::instruction::Instruction, solana_program::program_error::ProgramError> {
    // MintTo is instruction 7 in both v1 and v2
    // Format: [instruction_type (1 byte)] + [amount (8 bytes LE)]
    let mut data = Vec::with_capacity(9);
    data.push(7u8); // MintTo instruction
    data.extend_from_slice(&amount.to_le_bytes());
    
    Ok(solana_program::instruction::Instruction {
        program_id: *token_program_id,
        accounts: vec![
            solana_program::instruction::AccountMeta::new(*mint, false),
            solana_program::instruction::AccountMeta::new(*destination, false),
            solana_program::instruction::AccountMeta::new_readonly(*authority, true),
        ],
        data,
    })
}

/// Create a Burn instruction (works for both v1 and v2)
pub fn create_burn_instruction(
    token_program_id: &Pubkey,
    account: &Pubkey,
    mint: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Result<solana_program::instruction::Instruction, solana_program::program_error::ProgramError> {
    // Burn is instruction 8 in both v1 and v2
    // Format: [instruction_type (1 byte)] + [amount (8 bytes LE)]
    let mut data = Vec::with_capacity(9);
    data.push(8u8); // Burn instruction
    data.extend_from_slice(&amount.to_le_bytes());
    
    Ok(solana_program::instruction::Instruction {
        program_id: *token_program_id,
        accounts: vec![
            solana_program::instruction::AccountMeta::new(*account, false),
            solana_program::instruction::AccountMeta::new(*mint, false),
            solana_program::instruction::AccountMeta::new_readonly(*authority, true),
        ],
        data,
    })
}

/// Initialize a token account with dynamic program support
pub fn initialize_account<'a>(
    token_program: &AccountInfo<'a>,
    account: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    owner: &Pubkey,
    signer_seeds: Option<&[&[u8]]>,
) -> ProgramResult {
    let ix = create_initialize_account3_instruction(
        token_program.key,
        account.key,
        mint.key,
        owner,
    )?;

    let account_infos = vec![account.clone(), mint.clone()];

    if let Some(seeds) = signer_seeds {
        invoke_signed(&ix, &account_infos, &[seeds])
    } else {
        invoke(&ix, &account_infos)
    }
}

/// Transfer tokens with dynamic program support
pub fn transfer<'a>(
    token_program: &AccountInfo<'a>,
    source: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    signer_seeds: Option<&[&[u8]]>,
) -> ProgramResult {
    let ix = create_transfer_instruction(
        token_program.key,
        source.key,
        destination.key,
        authority.key,
        amount,
    )?;

    let account_infos = vec![
        source.clone(),
        destination.clone(),
        authority.clone(),
    ];

    if let Some(seeds) = signer_seeds {
        invoke_signed(&ix, &account_infos, &[seeds])
    } else {
        invoke(&ix, &account_infos)
    }
}

/// Mint tokens with dynamic program support
pub fn mint_to<'a>(
    token_program: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    signer_seeds: Option<&[&[u8]]>,
) -> ProgramResult {
    let ix = create_mint_to_instruction(
        token_program.key,
        mint.key,
        destination.key,
        authority.key,
        amount,
    )?;

    let account_infos = vec![
        mint.clone(),
        destination.clone(),
        authority.clone(),
    ];

    if let Some(seeds) = signer_seeds {
        invoke_signed(&ix, &account_infos, &[seeds])
    } else {
        invoke(&ix, &account_infos)
    }
}

/// Burn tokens with dynamic program support
pub fn burn<'a>(
    token_program: &AccountInfo<'a>,
    account: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    signer_seeds: Option<&[&[u8]]>,
) -> ProgramResult {
    let ix = create_burn_instruction(
        token_program.key,
        account.key,
        mint.key,
        authority.key,
        amount,
    )?;

    let account_infos = vec![
        account.clone(),
        mint.clone(),
        authority.clone(),
    ];

    if let Some(seeds) = signer_seeds {
        invoke_signed(&ix, &account_infos, &[seeds])
    } else {
        invoke(&ix, &account_infos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_token_program() {
        assert!(is_valid_token_program(&TOKEN_PROGRAM_V1));
        assert!(is_valid_token_program(&TOKEN_PROGRAM_V2));
        assert!(!is_valid_token_program(&Pubkey::default()));
    }

    #[test]
    fn test_get_token_account_size() {
        assert_eq!(get_token_account_size(&TOKEN_PROGRAM_V1), 165);
        assert_eq!(get_token_account_size(&TOKEN_PROGRAM_V2), 165);
    }

    #[test]
    fn test_get_mint_size() {
        assert_eq!(get_mint_size(&TOKEN_PROGRAM_V1), 82);
        assert_eq!(get_mint_size(&TOKEN_PROGRAM_V2), 82);
    }
}
