//! Fund Program Error Types
//! 
//! Defines all error types for the Fund Program.

use solana_program::program_error::ProgramError;
use thiserror::Error;

/// Fund Program errors
#[derive(Error, Debug, Copy, Clone)]
pub enum FundError {
    // === 授权错误 (0-9) ===
    
    /// [0] Unauthorized - caller is not the fund manager
    #[error("Unauthorized: caller is not the fund manager")]
    Unauthorized,
    
    /// [1] Not fund manager
    #[error("Not fund manager")]
    NotFundManager,
    
    /// [2] Not LP investor
    #[error("Not LP investor")]
    NotLPInvestor,
    
    /// [3] Admin required
    #[error("Admin required for this operation")]
    AdminRequired,
    
    /// [4] Unauthorized caller - CPI from unauthorized program
    #[error("Unauthorized caller: must be called by authorized program")]
    UnauthorizedCaller,
    
    // === 账户错误 (10-19) ===
    
    /// [10] Fund already initialized
    #[error("Fund is already initialized")]
    FundAlreadyInitialized,
    
    /// [11] Fund not initialized
    #[error("Fund is not initialized")]
    FundNotInitialized,
    
    /// [12] Invalid fund account
    #[error("Invalid fund account")]
    InvalidFundAccount,
    
    /// [13] LP position not found
    #[error("LP position not found")]
    LPPositionNotFound,
    
    /// [14] LP position already exists
    #[error("LP position already exists")]
    LPPositionAlreadyExists,
    
    /// [15] Invalid account owner
    #[error("Invalid account owner")]
    InvalidAccountOwner,
    
    /// [16] Invalid mint
    #[error("Invalid mint account")]
    InvalidMint,
    
    // === 资金错误 (20-29) ===
    
    /// [20] Insufficient balance
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    /// [21] Insufficient shares
    #[error("Insufficient shares for redemption")]
    InsufficientShares,
    
    /// [22] Deposit amount too small
    #[error("Deposit amount is below minimum")]
    DepositTooSmall,
    
    /// [23] Withdrawal would leave fund empty
    #[error("Cannot withdraw entire fund balance")]
    CannotEmptyFund,
    
    /// [24] Invalid amount
    #[error("Invalid amount: must be greater than zero")]
    InvalidAmount,
    
    // === 状态错误 (30-39) ===
    
    /// [30] Fund is closed
    #[error("Fund is closed for new deposits")]
    FundClosed,
    
    /// [31] Fund has open positions
    #[error("Fund has open positions, cannot close")]
    FundHasOpenPositions,
    
    /// [32] Fund is paused
    #[error("Fund is paused")]
    FundPaused,
    
    /// [33] Cannot close fund with LP positions
    #[error("Cannot close fund while LP positions exist")]
    FundHasLPPositions,
    
    /// [34] Fund name too long
    #[error("Fund name exceeds maximum length")]
    FundNameTooLong,
    
    // === 费用错误 (40-49) ===
    
    /// [40] Invalid fee configuration
    #[error("Invalid fee configuration")]
    InvalidFeeConfig,
    
    /// [41] Management fee too high
    #[error("Management fee exceeds maximum (10%)")]
    ManagementFeeTooHigh,
    
    /// [42] Performance fee too high
    #[error("Performance fee exceeds maximum (50%)")]
    PerformanceFeeTooHigh,
    
    /// [43] Fee collection too early
    #[error("Fee collection interval not reached")]
    FeeCollectionTooEarly,
    
    /// [44] No fees to collect
    #[error("No fees available to collect")]
    NoFeesToCollect,
    
    // === 计算错误 (50-59) ===
    
    /// [50] Overflow error
    #[error("Arithmetic overflow")]
    Overflow,
    
    /// [51] Underflow error
    #[error("Arithmetic underflow")]
    Underflow,
    
    /// [52] Division by zero
    #[error("Division by zero")]
    DivisionByZero,
    
    /// [53] NAV calculation error
    #[error("NAV calculation error")]
    NAVCalculationError,
    
    /// [54] Share calculation error
    #[error("Share calculation error")]
    ShareCalculationError,
    
    // === PDA 错误 (60-69) ===
    
    /// [60] Invalid PDA
    #[error("Invalid PDA derivation")]
    InvalidPDA,
    
    /// [61] Invalid seeds
    #[error("Invalid seeds for PDA")]
    InvalidSeeds,
    
    /// [62] PDA mismatch
    #[error("PDA does not match expected address")]
    PDAMismatch,
    
    // === Insurance Fund 错误 (70-89) ===
    
    /// [70] Insurance Fund already initialized
    #[error("Insurance Fund is already initialized")]
    InsuranceFundAlreadyInitialized,
    
    /// [71] Insurance Fund not initialized
    #[error("Insurance Fund is not initialized")]
    InsuranceFundNotInitialized,
    
    /// [72] Insurance Fund insufficient balance
    #[error("Insurance Fund has insufficient balance to cover shortfall")]
    InsuranceFundInsufficientBalance,
    
    /// [73] ADL in progress - redemptions paused
    #[error("ADL in progress: LP redemptions are temporarily paused")]
    ADLInProgress,
    
    /// [74] ADL not required
    #[error("ADL not required: Insurance Fund balance sufficient")]
    ADLNotRequired,
    
    /// [75] Invalid Insurance Fund config
    #[error("Invalid Insurance Fund configuration")]
    InvalidInsuranceFundConfig,
    
    /// [76] Snapshot too recent
    #[error("Hourly snapshot update too recent")]
    SnapshotTooRecent,
    
    /// [77] Withdrawal delay not met
    #[error("Withdrawal delay period not met")]
    WithdrawalDelayNotMet,
    
    // === Square Platform 错误 (90-99) ===
    
    /// [90] Invalid payment type
    #[error("Invalid payment type: must be 0 (KnowledgePurchase), 1 (Subscription), or 2 (LiveDonation)")]
    InvalidPaymentType,
    
    /// [91] Payment record already exists
    #[error("Payment record already exists for this transaction")]
    PaymentRecordAlreadyExists,
    
    /// [92] Invalid fee configuration
    #[error("Invalid fee configuration: creator share must be <= 10000 bps")]
    InvalidFeeConfiguration,
}

impl From<FundError> for ProgramError {
    fn from(e: FundError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let err = FundError::Unauthorized;
        let prog_err: ProgramError = err.into();
        assert_eq!(prog_err, ProgramError::Custom(0));
        
        // InsufficientBalance is the 13th enum variant (0-indexed = 12)
        // Variants: Unauthorized(0), NotFundManager(1), NotLPInvestor(2), AdminRequired(3),
        // UnauthorizedCaller(4), FundAlreadyInitialized(5), FundNotInitialized(6),
        // InvalidFundAccount(7), LPPositionNotFound(8), LPPositionAlreadyExists(9),
        // InvalidAccountOwner(10), InvalidMint(11), InsufficientBalance(12)
        let err = FundError::InsufficientBalance;
        let prog_err: ProgramError = err.into();
        assert_eq!(prog_err, ProgramError::Custom(12));
    }
}

