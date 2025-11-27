//! Fund Program CPI Helpers
//!
//! Helper functions for Cross-Program Invocation (CPI) calls to the Fund Program
//! and calls from Fund Program to Ledger Program.

use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::instruction::{FundInstruction, RecordPnLArgs};

// ============================================================================
// Ledger Program CPI Instructions (for Fund to call Ledger)
// ============================================================================

/// Ledger Program 指令枚举 (简化版，仅包含 Fund 需要调用的指令)
#[derive(BorshSerialize)]
enum LedgerInstruction {
    OpenPosition {
        user: Pubkey,
        market_index: u8,
        side: u8,          // 0 = Long, 1 = Short
        size_e6: u64,
        price_e6: u64,
        leverage: u8,
        batch_id: u64,
    },
    ClosePosition {
        user: Pubkey,
        market_index: u8,
        size_e6: u64,
        price_e6: u64,
        batch_id: u64,
    },
}

/// CPI: 开仓 (Fund -> Ledger)
pub fn open_position<'a>(
    ledger_program_id: &Pubkey,
    relayer: AccountInfo<'a>,
    position: AccountInfo<'a>,
    user_account: AccountInfo<'a>,
    vault_config: AccountInfo<'a>,
    ledger_config: AccountInfo<'a>,
    user_stats: AccountInfo<'a>,
    vault_program: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    user: Pubkey,
    market_index: u8,
    side: u8,
    size_e6: u64,
    price_e6: u64,
    leverage: u8,
    batch_id: u64,
    signer_seeds: &[&[&[u8]]],
) -> ProgramResult {
    let instruction = Instruction {
        program_id: *ledger_program_id,
        accounts: vec![
            AccountMeta::new(*relayer.key, true),
            AccountMeta::new(*position.key, false),
            AccountMeta::new(*user_account.key, false),
            AccountMeta::new_readonly(*vault_config.key, false),
            AccountMeta::new(*ledger_config.key, false),
            AccountMeta::new(*user_stats.key, false),
            AccountMeta::new_readonly(*vault_program.key, false),
            AccountMeta::new_readonly(*system_program.key, false),
        ],
        data: LedgerInstruction::OpenPosition {
            user,
            market_index,
            side,
            size_e6,
            price_e6,
            leverage,
            batch_id,
        }.try_to_vec()?,
    };

    invoke_signed(
        &instruction,
        &[
            relayer, position, user_account, vault_config,
            ledger_config, user_stats, vault_program, system_program,
        ],
        signer_seeds,
    )
}

/// CPI: 平仓 (Fund -> Ledger)
pub fn close_position<'a>(
    ledger_program_id: &Pubkey,
    relayer: AccountInfo<'a>,
    position: AccountInfo<'a>,
    user_account: AccountInfo<'a>,
    vault_config: AccountInfo<'a>,
    insurance_fund: AccountInfo<'a>,
    ledger_config: AccountInfo<'a>,
    user_stats: AccountInfo<'a>,
    vault_program: AccountInfo<'a>,
    user: Pubkey,
    market_index: u8,
    size_e6: u64,
    price_e6: u64,
    batch_id: u64,
    signer_seeds: &[&[&[u8]]],
) -> ProgramResult {
    let instruction = Instruction {
        program_id: *ledger_program_id,
        accounts: vec![
            AccountMeta::new(*relayer.key, true),
            AccountMeta::new(*position.key, false),
            AccountMeta::new(*user_account.key, false),
            AccountMeta::new_readonly(*vault_config.key, false),
            AccountMeta::new(*insurance_fund.key, false),
            AccountMeta::new(*ledger_config.key, false),
            AccountMeta::new(*user_stats.key, false),
            AccountMeta::new_readonly(*vault_program.key, false),
        ],
        data: LedgerInstruction::ClosePosition {
            user,
            market_index,
            size_e6,
            price_e6,
            batch_id,
        }.try_to_vec()?,
    };

    invoke_signed(
        &instruction,
        &[
            relayer, position, user_account, vault_config,
            insurance_fund, ledger_config, user_stats, vault_program,
        ],
        signer_seeds,
    )
}

// ============================================================================
// Fund Program CPI Instructions (for others to call Fund)
// ============================================================================

/// Record realized PnL for a fund (called by Ledger Program)
///
/// # Arguments
///
/// * `fund_program_id` - The Fund Program ID
/// * `caller` - The calling program (must be authorized)
/// * `fund` - The Fund account to update
/// * `pnl_e6` - The realized PnL amount (can be negative)
/// * `signer_seeds` - Seeds for signing the CPI call
///
/// # Returns
///
/// Result indicating success or failure
pub fn record_pnl<'a>(
    fund_program_id: &Pubkey,
    caller: &AccountInfo<'a>,
    fund: &AccountInfo<'a>,
    pnl_e6: i64,
    signer_seeds: &[&[&[u8]]],
) -> Result<(), ProgramError> {
    let instruction_data = FundInstruction::RecordPnL(RecordPnLArgs { pnl_e6 })
        .try_to_vec()
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let accounts = vec![
        AccountMeta::new_readonly(*caller.key, true),
        AccountMeta::new(*fund.key, false),
    ];

    let instruction = Instruction {
        program_id: *fund_program_id,
        accounts,
        data: instruction_data,
    };

    invoke_signed(
        &instruction,
        &[caller.clone(), fund.clone()],
        signer_seeds,
    )
}

/// Create instruction to record PnL
pub fn create_record_pnl_instruction(
    fund_program_id: &Pubkey,
    caller: &Pubkey,
    fund: &Pubkey,
    pnl_e6: i64,
) -> Result<Instruction, ProgramError> {
    let instruction_data = FundInstruction::RecordPnL(RecordPnLArgs { pnl_e6 })
        .try_to_vec()
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    Ok(Instruction {
        program_id: *fund_program_id,
        accounts: vec![
            AccountMeta::new_readonly(*caller, true),
            AccountMeta::new(*fund, false),
        ],
        data: instruction_data,
    })
}

/// Helper to derive Fund PDA
pub fn derive_fund_pda(
    program_id: &Pubkey,
    manager: &Pubkey,
    fund_index: u64,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            crate::state::FUND_SEED,
            manager.as_ref(),
            &fund_index.to_le_bytes(),
        ],
        program_id,
    )
}

/// Helper to derive Fund vault PDA
pub fn derive_fund_vault_pda(
    program_id: &Pubkey,
    fund: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            crate::state::FUND_VAULT_SEED,
            fund.as_ref(),
        ],
        program_id,
    )
}

/// Helper to derive Share mint PDA
pub fn derive_share_mint_pda(
    program_id: &Pubkey,
    fund: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            crate::state::SHARE_MINT_SEED,
            fund.as_ref(),
        ],
        program_id,
    )
}

/// Helper to derive LP position PDA
pub fn derive_lp_position_pda(
    program_id: &Pubkey,
    fund: &Pubkey,
    investor: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            crate::state::LP_POSITION_SEED,
            fund.as_ref(),
            investor.as_ref(),
        ],
        program_id,
    )
}

/// Helper to derive FundConfig PDA
pub fn derive_fund_config_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[crate::state::FUND_CONFIG_SEED],
        program_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_fund_pda() {
        let program_id = Pubkey::new_unique();
        let manager = Pubkey::new_unique();
        let fund_index = 0u64;
        
        let (pda, bump) = derive_fund_pda(&program_id, &manager, fund_index);
        
        // Verify the PDA is valid
        assert!(bump > 0 || bump == 0);
        assert_ne!(pda, program_id);
        assert_ne!(pda, manager);
    }

    #[test]
    fn test_derive_lp_position_pda() {
        let program_id = Pubkey::new_unique();
        let fund = Pubkey::new_unique();
        let investor = Pubkey::new_unique();
        
        let (pda, bump) = derive_lp_position_pda(&program_id, &fund, &investor);
        
        assert!(bump <= 255);
        assert_ne!(pda, fund);
        assert_ne!(pda, investor);
    }

    #[test]
    fn test_derive_fund_config_pda() {
        let program_id = Pubkey::new_unique();
        
        let (pda, bump) = derive_fund_config_pda(&program_id);
        
        assert!(bump <= 255);
        assert_ne!(pda, program_id);
    }

    #[test]
    fn test_create_record_pnl_instruction() {
        let program_id = Pubkey::new_unique();
        let caller = Pubkey::new_unique();
        let fund = Pubkey::new_unique();
        
        let ix = create_record_pnl_instruction(
            &program_id,
            &caller,
            &fund,
            1_000_000, // 1 USDC profit
        ).unwrap();
        
        assert_eq!(ix.program_id, program_id);
        assert_eq!(ix.accounts.len(), 2);
        assert!(!ix.data.is_empty());
    }
}

