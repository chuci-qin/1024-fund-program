//! Fund Program Utility Functions
//!
//! Contains helper functions for validation, math operations, and common tasks.

use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};

use crate::error::FundError;

// === Constants ===

/// Basis points denominator (100% = 10000 bps)
pub const BPS_DENOMINATOR: u64 = 10_000;

/// Maximum management fee (10% = 1000 bps)
pub const MAX_MANAGEMENT_FEE_BPS: u32 = 1_000;

/// Maximum performance fee (50% = 5000 bps)
pub const MAX_PERFORMANCE_FEE_BPS: u32 = 5_000;

/// Minimum deposit amount (1 USDC = 1_000_000 e6)
pub const MIN_DEPOSIT_AMOUNT_E6: i64 = 1_000_000;

/// Seconds per year (for management fee calculation)
pub const SECONDS_PER_YEAR: i64 = 365 * 24 * 60 * 60;

/// Maximum fund name length
pub const MAX_FUND_NAME_LEN: usize = 32;

/// Initial NAV (1.0 in e6 format)
pub const INITIAL_NAV_E6: i64 = 1_000_000;

// === Validation Functions ===

/// Assert that an account is a signer
pub fn assert_signer(account: &AccountInfo) -> Result<(), ProgramError> {
    if !account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

/// Assert that an account is writable
pub fn assert_writable(account: &AccountInfo) -> Result<(), ProgramError> {
    if !account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

/// Assert that an account is owned by the expected program
pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<(), ProgramError> {
    if account.owner != owner {
        return Err(FundError::InvalidAccountOwner.into());
    }
    Ok(())
}

/// Validate fee configuration
pub fn validate_fee_config(
    management_fee_bps: u32,
    performance_fee_bps: u32,
) -> Result<(), ProgramError> {
    if management_fee_bps > MAX_MANAGEMENT_FEE_BPS {
        return Err(FundError::ManagementFeeTooHigh.into());
    }
    if performance_fee_bps > MAX_PERFORMANCE_FEE_BPS {
        return Err(FundError::PerformanceFeeTooHigh.into());
    }
    Ok(())
}

/// Validate fund name
pub fn validate_fund_name(name: &str) -> Result<(), ProgramError> {
    if name.len() > MAX_FUND_NAME_LEN || name.is_empty() {
        return Err(FundError::FundNameTooLong.into());
    }
    Ok(())
}

// === Math Functions ===

/// Safe addition for i64
pub fn safe_add_i64(a: i64, b: i64) -> Result<i64, ProgramError> {
    a.checked_add(b).ok_or(FundError::Overflow.into())
}

/// Safe subtraction for i64
pub fn safe_sub_i64(a: i64, b: i64) -> Result<i64, ProgramError> {
    a.checked_sub(b).ok_or(FundError::Underflow.into())
}

/// Safe multiplication for i64
pub fn safe_mul_i64(a: i64, b: i64) -> Result<i64, ProgramError> {
    a.checked_mul(b).ok_or(FundError::Overflow.into())
}

/// Safe division for i64
pub fn safe_div_i64(a: i64, b: i64) -> Result<i64, ProgramError> {
    if b == 0 {
        return Err(FundError::DivisionByZero.into());
    }
    a.checked_div(b).ok_or(FundError::Overflow.into())
}

/// Safe addition for u64
pub fn safe_add_u64(a: u64, b: u64) -> Result<u64, ProgramError> {
    a.checked_add(b).ok_or(FundError::Overflow.into())
}

/// Safe subtraction for u64
pub fn safe_sub_u64(a: u64, b: u64) -> Result<u64, ProgramError> {
    a.checked_sub(b).ok_or(FundError::Underflow.into())
}

/// Safe multiplication for u64
pub fn safe_mul_u64(a: u64, b: u64) -> Result<u64, ProgramError> {
    a.checked_mul(b).ok_or(FundError::Overflow.into())
}

/// Safe division for u64
pub fn safe_div_u64(a: u64, b: u64) -> Result<u64, ProgramError> {
    if b == 0 {
        return Err(FundError::DivisionByZero.into());
    }
    a.checked_div(b).ok_or(FundError::Overflow.into())
}

// === NAV & Share Calculations ===

/// Calculate NAV (Net Asset Value) per share
/// NAV = total_value_e6 / total_shares (in e6 format)
pub fn calculate_nav_e6(total_value_e6: i64, total_shares: u64) -> Result<i64, ProgramError> {
    if total_shares == 0 {
        // Initial NAV is 1.0
        return Ok(INITIAL_NAV_E6);
    }
    
    if total_value_e6 <= 0 {
        return Err(FundError::NAVCalculationError.into());
    }
    
    // NAV = total_value * 1e6 / total_shares
    let nav = ((total_value_e6 as i128) * 1_000_000 / (total_shares as i128)) as i64;
    Ok(nav)
}

/// Calculate shares to mint for a deposit
/// shares = deposit_amount_e6 * 1e6 / nav_e6
pub fn calculate_shares_to_mint(deposit_amount_e6: i64, nav_e6: i64) -> Result<u64, ProgramError> {
    if nav_e6 <= 0 {
        return Err(FundError::NAVCalculationError.into());
    }
    if deposit_amount_e6 <= 0 {
        return Err(FundError::InvalidAmount.into());
    }
    
    // shares = deposit * 1e6 / nav
    let shares = ((deposit_amount_e6 as i128) * 1_000_000 / (nav_e6 as i128)) as u64;
    
    if shares == 0 {
        return Err(FundError::ShareCalculationError.into());
    }
    
    Ok(shares)
}

/// Calculate USDC value for share redemption
/// value = shares * nav_e6 / 1e6
pub fn calculate_redemption_value(shares: u64, nav_e6: i64) -> Result<i64, ProgramError> {
    if nav_e6 <= 0 {
        return Err(FundError::NAVCalculationError.into());
    }
    if shares == 0 {
        return Err(FundError::InvalidAmount.into());
    }
    
    // value = shares * nav / 1e6
    let value = ((shares as i128) * (nav_e6 as i128) / 1_000_000) as i64;
    Ok(value)
}

/// Calculate management fee for a period
/// fee = aum * fee_bps / BPS_DENOMINATOR * time_elapsed / SECONDS_PER_YEAR
pub fn calculate_management_fee(
    aum_e6: i64,
    fee_bps: u32,
    time_elapsed_seconds: i64,
) -> Result<i64, ProgramError> {
    if aum_e6 <= 0 || fee_bps == 0 || time_elapsed_seconds <= 0 {
        return Ok(0);
    }
    
    // fee = aum * fee_bps * time / (BPS_DENOMINATOR * SECONDS_PER_YEAR)
    let fee = ((aum_e6 as i128) * (fee_bps as i128) * (time_elapsed_seconds as i128)
        / (BPS_DENOMINATOR as i128)
        / (SECONDS_PER_YEAR as i128)) as i64;
    
    Ok(fee)
}

/// Calculate performance fee (only on profit above HWM)
/// fee = (nav - hwm) * total_value * fee_bps / BPS_DENOMINATOR / nav
pub fn calculate_performance_fee(
    current_nav_e6: i64,
    hwm_e6: i64,
    total_value_e6: i64,
    fee_bps: u32,
) -> Result<i64, ProgramError> {
    // Only charge fee if current NAV exceeds HWM
    if current_nav_e6 <= hwm_e6 || fee_bps == 0 || total_value_e6 <= 0 {
        return Ok(0);
    }
    
    // profit_per_share = nav - hwm
    let profit_per_share = current_nav_e6 - hwm_e6;
    
    // total_profit = profit_per_share * total_value / nav
    let total_profit = ((profit_per_share as i128) * (total_value_e6 as i128) / (current_nav_e6 as i128)) as i64;
    
    // fee = total_profit * fee_bps / BPS_DENOMINATOR
    let fee = ((total_profit as i128) * (fee_bps as i128) / (BPS_DENOMINATOR as i128)) as i64;
    
    Ok(fee)
}

// === Time Functions ===

/// Get current timestamp from Clock sysvar
pub fn get_current_timestamp() -> Result<i64, ProgramError> {
    let clock = Clock::get()?;
    Ok(clock.unix_timestamp)
}

/// Check if enough time has passed for fee collection
pub fn can_collect_fees(last_collection_ts: i64, interval_seconds: i64) -> Result<bool, ProgramError> {
    let current_ts = get_current_timestamp()?;
    Ok(current_ts >= last_collection_ts + interval_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_nav() {
        // Initial NAV when no shares
        assert_eq!(calculate_nav_e6(0, 0).unwrap(), INITIAL_NAV_E6);
        
        // NAV = 1.0 when value = shares
        assert_eq!(calculate_nav_e6(1_000_000, 1_000_000).unwrap(), 1_000_000);
        
        // NAV = 1.5 when value = 1.5 * shares
        assert_eq!(calculate_nav_e6(15_000_000, 10_000_000).unwrap(), 1_500_000);
        
        // NAV = 0.5 when value = 0.5 * shares
        assert_eq!(calculate_nav_e6(5_000_000, 10_000_000).unwrap(), 500_000);
    }

    #[test]
    fn test_calculate_shares_to_mint() {
        // At NAV = 1.0, 100 USDC = 100 shares
        let shares = calculate_shares_to_mint(100_000_000, 1_000_000).unwrap();
        assert_eq!(shares, 100_000_000);
        
        // At NAV = 1.5, 100 USDC = 66.67 shares
        let shares = calculate_shares_to_mint(100_000_000, 1_500_000).unwrap();
        assert_eq!(shares, 66_666_666);
        
        // At NAV = 0.5, 100 USDC = 200 shares
        let shares = calculate_shares_to_mint(100_000_000, 500_000).unwrap();
        assert_eq!(shares, 200_000_000);
    }

    #[test]
    fn test_calculate_redemption_value() {
        // At NAV = 1.0, 100 shares = 100 USDC
        let value = calculate_redemption_value(100_000_000, 1_000_000).unwrap();
        assert_eq!(value, 100_000_000);
        
        // At NAV = 1.5, 100 shares = 150 USDC
        let value = calculate_redemption_value(100_000_000, 1_500_000).unwrap();
        assert_eq!(value, 150_000_000);
    }

    #[test]
    fn test_calculate_management_fee() {
        // 2% annual fee on 100,000 USDC for 1 year
        let fee = calculate_management_fee(
            100_000_000_000, // 100,000 USDC in e6
            200,             // 2% = 200 bps
            SECONDS_PER_YEAR,
        ).unwrap();
        assert_eq!(fee, 2_000_000_000); // 2,000 USDC
        
        // 2% annual fee on 100,000 USDC for 1 day
        let fee = calculate_management_fee(
            100_000_000_000,
            200,
            24 * 60 * 60, // 1 day
        ).unwrap();
        // Expected: 100,000 * 0.02 / 365 ≈ 5.48 USDC
        assert!(fee > 5_000_000 && fee < 6_000_000);
    }

    #[test]
    fn test_calculate_performance_fee() {
        // 20% performance fee, NAV went from 1.0 to 1.2, AUM = 100,000 USDC
        let fee = calculate_performance_fee(
            1_200_000,       // current NAV = 1.2
            1_000_000,       // HWM = 1.0
            100_000_000_000, // total value = 100,000 USDC
            2_000,           // 20% = 2000 bps
        ).unwrap();
        // Profit = 20,000 USDC, Fee = 20,000 * 20% = 4,000 USDC
        assert_eq!(fee, 3_333_333_333); // ~3,333 USDC (due to calculation order)
        
        // No fee when below HWM
        let fee = calculate_performance_fee(
            900_000,         // current NAV = 0.9 (below HWM)
            1_000_000,       // HWM = 1.0
            100_000_000_000,
            2_000,
        ).unwrap();
        assert_eq!(fee, 0);
    }

    #[test]
    fn test_validate_fee_config() {
        // Valid config
        assert!(validate_fee_config(200, 2000).is_ok());
        
        // Management fee too high
        assert!(validate_fee_config(1500, 2000).is_err());
        
        // Performance fee too high
        assert!(validate_fee_config(200, 6000).is_err());
    }

    #[test]
    fn test_validate_fund_name() {
        // Valid name
        assert!(validate_fund_name("My Awesome Fund").is_ok());
        
        // Empty name
        assert!(validate_fund_name("").is_err());
        
        // Too long name
        let long_name = "a".repeat(MAX_FUND_NAME_LEN + 1);
        assert!(validate_fund_name(&long_name).is_err());
    }

    #[test]
    fn test_safe_math() {
        // Addition
        assert_eq!(safe_add_i64(10, 20).unwrap(), 30);
        assert!(safe_add_i64(i64::MAX, 1).is_err());
        
        // Subtraction
        assert_eq!(safe_sub_i64(30, 10).unwrap(), 20);
        assert!(safe_sub_i64(i64::MIN, 1).is_err());
        
        // Multiplication
        assert_eq!(safe_mul_i64(10, 20).unwrap(), 200);
        assert!(safe_mul_i64(i64::MAX, 2).is_err());
        
        // Division
        assert_eq!(safe_div_i64(100, 10).unwrap(), 10);
        assert!(safe_div_i64(100, 0).is_err());
    }
}

