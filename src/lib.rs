//! 1024 DEX Fund Program
//!
//! This program enables users to create and manage investment funds on 1024 DEX.
//! Fund managers can create funds, accept LP investments, trade with fund assets,
//! and collect management and performance fees.
//!
//! ## Features
//!
//! - **Fund Creation**: Create funds with customizable fee structures
//! - **LP Investment**: Allow LP investors to deposit and redeem
//! - **Share Tokens**: Mint/burn share tokens representing fund ownership
//! - **NAV Calculation**: Track Net Asset Value per share
//! - **Fee Collection**: Collect management fees (time-based) and performance fees (profit-based)
//! - **High Water Mark**: Ensure performance fees only on new profits
//!
//! ## Account Types
//!
//! - `FundConfig`: Global program configuration
//! - `Fund`: Individual fund account with stats and settings
//! - `LPPosition`: LP investor's position in a fund
//!
//! ## Instructions
//!
//! - `Initialize`: Initialize the Fund Program
//! - `CreateFund`: Create a new fund
//! - `DepositToFund`: LP deposits USDC, receives shares
//! - `RedeemFromFund`: LP redeems shares for USDC
//! - `CollectFees`: Manager collects accrued fees
//! - `TradeFund`: Manager trades using fund assets
//!
//! ## CPI Integration
//!
//! This program integrates with:
//! - Vault Program: For USDC custody
//! - Ledger Program: For trading operations

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub mod cpi;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

/// Program entrypoint
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    processor::process_instruction(program_id, accounts, instruction_data)
}

// Re-export commonly used items
pub use error::FundError;
pub use instruction::FundInstruction;
pub use state::{Fund, FundConfig, FundStats, FeeConfig, LPPosition};

// Program ID placeholder - will be replaced after deployment
solana_program::declare_id!("FundProg11111111111111111111111111111111111");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_id() {
        // Verify program ID is valid
        assert!(crate::id().to_string().len() > 0);
    }
}

