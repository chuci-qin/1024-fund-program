//! Fund Program CPI Helpers
//!
//! Helper functions for Cross-Program Invocation (CPI) calls to the Fund Program
//! and calls from Fund Program to Ledger Program.

use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
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
// Vault Program CPI Instructions (for Fund to call Vault)
// ============================================================================

/// Vault Program 指令枚举 (简化版，仅包含 Fund 需要调用的指令)
///
/// 必须与 1024-vault-program/src/instruction.rs 中的 VaultInstruction 枚举
/// 保持 Borsh 序列化顺序**完全一致**。
///
/// 审计日期: 2026-02-08
/// 逐一比对确认 42 个变体顺序与 Vault 程序完全对齐。
#[derive(BorshSerialize)]
#[allow(dead_code)]
enum VaultInstruction {
    // 0: Initialize
    _Initialize { _ledger_program: Pubkey, _delegation_program: Pubkey, _fund_program: Pubkey },
    // 1: InitializeUser
    _InitializeUser,
    // 2: Deposit
    _Deposit { _amount: u64 },
    // 3: Withdraw
    _Withdraw { _amount: u64 },
    // 4: LockMargin
    _LockMargin { _amount: u64 },
    // 5: ReleaseMargin
    _ReleaseMargin { _amount: u64 },
    // 6: ClosePositionSettle
    _ClosePositionSettle { _margin_to_release: u64, _realized_pnl: i64, _fee: u64 },
    // 7: LiquidatePosition (注意: Vault 有 3 个字段 margin/user_remainder/liquidation_penalty)
    _LiquidatePosition { _margin: u64, _user_remainder: u64, _liquidation_penalty: u64 },
    // 8: AddAuthorizedCaller
    _AddAuthorizedCaller { _caller: Pubkey },
    // 9: RemoveAuthorizedCaller
    _RemoveAuthorizedCaller { _caller: Pubkey },
    // 10: SetPaused
    _SetPaused { _paused: bool },
    // 11: UpdateAdmin
    _UpdateAdmin { _new_admin: Pubkey },
    // 12: SetFundProgram
    _SetFundProgram { _fund_program: Pubkey },
    // 13: SetLedgerProgram
    _SetLedgerProgram { _ledger_program: Pubkey },
    // 14: AdminForceReleaseMargin
    _AdminForceReleaseMargin { _amount: u64 },
    // ===== Prediction Market 指令 (15-24) =====
    // 15: InitializePredictionMarketUser
    _InitializePredictionMarketUser,
    // 16: PredictionMarketLock
    _PredictionMarketLock { _amount: u64 },
    // 17: PredictionMarketUnlock
    _PredictionMarketUnlock { _amount: u64 },
    // 18: PredictionMarketSettle
    _PredictionMarketSettle { _locked_amount: u64, _settlement_amount: u64 },
    // 19: PredictionMarketClaimSettlement
    _PredictionMarketClaimSettlement,
    // 20: AdminPredictionMarketForceUnlock
    _AdminPredictionMarketForceUnlock { _amount: u64 },
    // 21: PredictionMarketLockWithFee
    _PredictionMarketLockWithFee { _gross_amount: u64 },
    // 22: PredictionMarketUnlockWithFee
    _PredictionMarketUnlockWithFee { _gross_amount: u64 },
    // 23: PredictionMarketTradeWithFee
    _PredictionMarketTradeWithFee { _trade_amount: u64, _is_taker: bool },
    // 24: PredictionMarketSettleWithFee
    _PredictionMarketSettleWithFee { _locked_amount: u64, _settlement_amount: u64 },
    // ===== Relayer 指令 (25-26) =====
    // 25: RelayerDeposit ← 用于 Fund Redeem (增加用户 Vault 余额)
    RelayerDeposit { user_wallet: Pubkey, amount: u64 },
    // 26: RelayerWithdraw ← 用于 Fund Deposit (扣减用户 Vault 余额)
    RelayerWithdraw { user_wallet: Pubkey, amount: u64 },
    // ===== Spot 指令 (27-37) =====
    // 27: InitializeSpotUser
    _InitializeSpotUser,
    // 28: SpotDeposit
    _SpotDeposit { _token_index: u16, _amount: u64 },
    // 29: SpotWithdraw
    _SpotWithdraw { _token_index: u16, _amount: u64 },
    // 30: SpotLockBalance
    _SpotLockBalance { _token_index: u16, _amount: u64 },
    // 31: SpotUnlockBalance
    _SpotUnlockBalance { _token_index: u16, _amount: u64 },
    // 32: SpotSettleTrade
    _SpotSettleTrade { _is_buy: bool, _base_token_index: u16, _quote_token_index: u16, _base_amount: u64, _quote_amount: u64, _sequence: u64 },
    // 33: RelayerSpotDeposit
    _RelayerSpotDeposit { _user_wallet: Pubkey, _token_index: u16, _amount: u64 },
    // 34: RelayerSpotWithdraw
    _RelayerSpotWithdraw { _user_wallet: Pubkey, _token_index: u16, _amount: u64 },
    // 35: RelayerSpotSettleTrade
    _RelayerSpotSettleTrade { _maker_wallet: Pubkey, _taker_wallet: Pubkey, _base_token_index: u16, _quote_token_index: u16, _base_amount_e6: i64, _quote_amount_e6: i64, _maker_fee_e6: i64, _taker_fee_e6: i64, _taker_is_buy: bool, _sequence: u64 },
    // 36: SpotAllocateFromVault
    _SpotAllocateFromVault { _user_wallet: Pubkey, _amount: u64 },
    // 37: SpotReleaseToVault
    _SpotReleaseToVault { _user_wallet: Pubkey, _amount: u64 },
    // ===== 站内支付 (38-41) =====
    // 38: RelayerInternalTransfer
    _RelayerInternalTransfer { _from_wallet: Pubkey, _to_wallet: Pubkey, _amount: u64, _fee: u64, _transfer_type: u8, _reference_hash: [u8; 32] },
    // 39: InitRecurringAuth
    _InitRecurringAuth { _payer: Pubkey, _payee: Pubkey, _amount: u64, _interval_seconds: i64, _max_cycles: u32, _registration_fee: u64 },
    // 40: ExecuteRecurringPayment
    _ExecuteRecurringPayment { _payer: Pubkey, _payee: Pubkey, _amount: u64, _fee: u64, _cycle_count: u32 },
    // 41: CancelRecurringAuth
    _CancelRecurringAuth { _payer: Pubkey, _payee: Pubkey },
}

/// CPI: Vault RelayerWithdraw (Fund deposit → reduce user's Vault balance)
///
/// When a user deposits into a Fund, their Vault voucher balance decreases.
/// The relayer (who is also the Vault admin) signs this transaction.
pub fn vault_relayer_withdraw<'a>(
    vault_program_id: &Pubkey,
    relayer: AccountInfo<'a>,          // [signer] Admin/Relayer
    user_account: AccountInfo<'a>,     // [writable] UserAccount PDA
    vault_config: AccountInfo<'a>,     // [] VaultConfig
    user_wallet: Pubkey,
    amount: u64,
) -> ProgramResult {
    let instruction = Instruction {
        program_id: *vault_program_id,
        accounts: vec![
            AccountMeta::new(*relayer.key, true),
            AccountMeta::new(*user_account.key, false),
            AccountMeta::new_readonly(*vault_config.key, false),
        ],
        data: VaultInstruction::RelayerWithdraw { user_wallet, amount }
            .try_to_vec()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    };

    invoke(
        &instruction,
        &[relayer, user_account, vault_config],
    )
}

/// CPI: Vault RelayerDeposit (Fund redeem → increase user's Vault balance)
///
/// When a user redeems from a Fund, their Vault voucher balance increases.
/// The relayer (who is also the Vault admin) signs this transaction.
pub fn vault_relayer_deposit<'a>(
    vault_program_id: &Pubkey,
    relayer: AccountInfo<'a>,          // [signer] Admin/Relayer
    user_account: AccountInfo<'a>,     // [writable] UserAccount PDA
    vault_config: AccountInfo<'a>,     // [writable] VaultConfig
    system_program: AccountInfo<'a>,   // [] System Program (for auto-init)
    user_wallet: Pubkey,
    amount: u64,
) -> ProgramResult {
    let instruction = Instruction {
        program_id: *vault_program_id,
        accounts: vec![
            AccountMeta::new(*relayer.key, true),
            AccountMeta::new(*user_account.key, false),
            AccountMeta::new(*vault_config.key, false),
            AccountMeta::new_readonly(*system_program.key, false),
        ],
        data: VaultInstruction::RelayerDeposit { user_wallet, amount }
            .try_to_vec()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    };

    invoke(
        &instruction,
        &[relayer, user_account, vault_config, system_program],
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
/// * `caller` - The calling program (must be authorized Ledger Program)
/// * `fund` - The Fund account to update
/// * `fund_config` - The FundConfig PDA (needed for caller verification)
/// * `pnl_e6` - The realized PnL amount (can be negative)
/// * `signer_seeds` - Seeds for signing the CPI call
pub fn record_pnl<'a>(
    fund_program_id: &Pubkey,
    caller: &AccountInfo<'a>,
    fund: &AccountInfo<'a>,
    fund_config: &AccountInfo<'a>,
    pnl_e6: i64,
    signer_seeds: &[&[&[u8]]],
) -> Result<(), ProgramError> {
    let instruction_data = FundInstruction::RecordPnL(RecordPnLArgs { pnl_e6 })
        .try_to_vec()
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // processor.rs process_record_pnl 需要 3 个账户: caller, fund, fund_config
    let accounts = vec![
        AccountMeta::new_readonly(*caller.key, true),
        AccountMeta::new(*fund.key, false),
        AccountMeta::new_readonly(*fund_config.key, false),
    ];

    let instruction = Instruction {
        program_id: *fund_program_id,
        accounts,
        data: instruction_data,
    };

    invoke_signed(
        &instruction,
        &[caller.clone(), fund.clone(), fund_config.clone()],
        signer_seeds,
    )
}

/// Create instruction to record PnL
pub fn create_record_pnl_instruction(
    fund_program_id: &Pubkey,
    caller: &Pubkey,
    fund: &Pubkey,
    fund_config: &Pubkey,
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
            AccountMeta::new_readonly(*fund_config, false),
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
        let fund_config = Pubkey::new_unique();
        
        let ix = create_record_pnl_instruction(
            &program_id,
            &caller,
            &fund,
            &fund_config,
            1_000_000, // 1 USDC profit
        ).unwrap();
        
        assert_eq!(ix.program_id, program_id);
        assert_eq!(ix.accounts.len(), 3); // caller + fund + fund_config
        assert!(!ix.data.is_empty());
    }
}

