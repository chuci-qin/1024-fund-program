//! Fund Program Instructions
//!
//! Defines all instructions for the Fund Program.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::state::FeeConfig;

/// All instructions supported by the Fund Program
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum FundInstruction {
    // === Initialization (0-9) ===
    
    /// Initialize the Fund Program config
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority (admin)
    /// 1. `[writable]` FundConfig PDA
    /// 2. `[]` Vault Program
    /// 3. `[]` Ledger Program
    /// 4. `[]` System Program
    Initialize(InitializeArgs),
    
    /// Create a new fund
    /// 
    /// Accounts:
    /// 0. `[signer]` Fund manager
    /// 1. `[writable]` Fund PDA
    /// 2. `[writable]` Fund vault PDA (token account)
    /// 3. `[writable]` Share mint PDA
    /// 4. `[writable]` FundConfig PDA
    /// 5. `[]` USDC mint
    /// 6. `[]` Token Program
    /// 7. `[]` System Program
    /// 8. `[]` Rent Sysvar
    CreateFund(CreateFundArgs),
    
    // === Fund Management (10-19) ===
    
    /// Update fund configuration
    /// 
    /// Accounts:
    /// 0. `[signer]` Fund manager
    /// 1. `[writable]` Fund PDA
    UpdateFund(UpdateFundArgs),
    
    /// Open/close fund for deposits
    /// 
    /// Accounts:
    /// 0. `[signer]` Fund manager
    /// 1. `[writable]` Fund PDA
    SetFundOpen(SetFundOpenArgs),
    
    /// Pause/unpause fund
    /// 
    /// Accounts:
    /// 0. `[signer]` Fund manager
    /// 1. `[writable]` Fund PDA
    SetFundPaused(SetFundPausedArgs),
    
    /// Close a fund (manager only)
    /// 
    /// Accounts:
    /// 0. `[signer]` Fund manager
    /// 1. `[writable]` Fund PDA
    /// 2. `[writable]` Fund vault PDA
    /// 3. `[writable]` Share mint PDA
    /// 4. `[writable]` FundConfig PDA
    /// 5. `[writable]` Manager's USDC account
    /// 6. `[]` Token Program
    CloseFund,
    
    // === LP Operations (20-29) ===
    
    /// Deposit USDC into a fund as LP
    /// 
    /// Accounts:
    /// 0. `[signer]` LP investor
    /// 1. `[writable]` Fund PDA
    /// 2. `[writable]` Fund vault PDA
    /// 3. `[writable]` LP's USDC account
    /// 4. `[writable]` LP Position PDA
    /// 5. `[writable]` LP's share token account
    /// 6. `[writable]` Share mint PDA
    /// 7. `[]` Token Program
    /// 8. `[]` System Program
    DepositToFund(DepositToFundArgs),
    
    /// Redeem shares from a fund
    /// 
    /// Accounts:
    /// 0. `[signer]` LP investor
    /// 1. `[writable]` Fund PDA
    /// 2. `[writable]` Fund vault PDA
    /// 3. `[writable]` LP's USDC account
    /// 4. `[writable]` LP Position PDA
    /// 5. `[writable]` LP's share token account
    /// 6. `[writable]` Share mint PDA
    /// 7. `[]` Token Program
    RedeemFromFund(RedeemFromFundArgs),
    
    // === Trading Operations (30-39) ===
    
    /// Trade using fund assets (manager only)
    /// 
    /// Accounts:
    /// 0. `[signer]` Fund manager
    /// 1. `[writable]` Fund PDA
    /// 2. `[]` Ledger Program
    /// 3. ... (Ledger Program required accounts)
    TradeFund(TradeFundArgs),
    
    /// Close a position for the fund (manager only)
    /// 
    /// Accounts:
    /// 0. `[signer]` Fund manager
    /// 1. `[writable]` Fund PDA
    /// 2. `[]` Ledger Program
    /// 3. ... (Ledger Program required accounts)
    CloseFundPosition(CloseFundPositionArgs),
    
    // === Fee Operations (40-49) ===
    
    /// Collect management and performance fees (manager only)
    /// 
    /// Accounts:
    /// 0. `[signer]` Fund manager
    /// 1. `[writable]` Fund PDA
    /// 2. `[writable]` Fund vault PDA
    /// 3. `[writable]` Manager's USDC account
    /// 4. `[]` Token Program
    CollectFees,
    
    // === Admin Operations (50-59) ===
    
    /// Update program authority
    /// 
    /// Accounts:
    /// 0. `[signer]` Current authority
    /// 1. `[writable]` FundConfig PDA
    /// 2. `[]` New authority
    UpdateAuthority(UpdateAuthorityArgs),
    
    /// Pause/unpause the entire program
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority
    /// 1. `[writable]` FundConfig PDA
    SetProgramPaused(SetProgramPausedArgs),
    
    // === NAV Operations (60-69) ===
    
    /// Update NAV for a fund (can be called by anyone)
    /// 
    /// Accounts:
    /// 0. `[writable]` Fund PDA
    UpdateNAV,
    
    /// Record realized PnL (called by Ledger Program via CPI)
    /// 
    /// Accounts:
    /// 0. `[signer]` Caller program (Ledger)
    /// 1. `[writable]` Fund PDA
    RecordPnL(RecordPnLArgs),
    
    // === Insurance Fund Operations (70-89) ===
    
    /// Initialize Insurance Fund
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority (admin)
    /// 1. `[writable]` Fund PDA (for Insurance Fund)
    /// 2. `[writable]` InsuranceFundConfig PDA
    /// 3. `[writable]` Fund vault PDA (token account)
    /// 4. `[writable]` Share mint PDA
    /// 5. `[writable]` FundConfig PDA
    /// 6. `[]` USDC mint
    /// 7. `[]` Token Program
    /// 8. `[]` System Program
    /// 9. `[]` Rent Sysvar
    InitializeInsuranceFund(InitializeInsuranceFundArgs),
    
    /// Add liquidation income to Insurance Fund (CPI from Ledger)
    /// 
    /// Accounts:
    /// 0. `[signer]` Caller program (Ledger)
    /// 1. `[writable]` Fund PDA
    /// 2. `[writable]` InsuranceFundConfig PDA
    /// 3. `[writable]` Fund vault PDA
    /// 4. `[]` Source token account (user or liquidation proceeds)
    /// 5. `[]` Token Program
    AddLiquidationIncome(AddLiquidationIncomeArgs),
    
    /// Add ADL profit to Insurance Fund (CPI from Ledger)
    /// 
    /// Accounts:
    /// 0. `[signer]` Caller program (Ledger)
    /// 1. `[writable]` Fund PDA
    /// 2. `[writable]` InsuranceFundConfig PDA
    AddADLProfit(AddADLProfitArgs),
    
    /// Cover shortfall from Insurance Fund (CPI from Ledger)
    /// Returns remaining shortfall if insurance fund insufficient
    /// 
    /// Accounts:
    /// 0. `[signer]` Caller program (Ledger)
    /// 1. `[writable]` Fund PDA
    /// 2. `[writable]` InsuranceFundConfig PDA
    /// 3. `[writable]` Fund vault PDA
    /// 4. `[writable]` Destination token account
    /// 5. `[]` Token Program
    CoverShortfall(CoverShortfallArgs),
    
    /// Update hourly snapshot (called by Relayer)
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority or Relayer
    /// 1. `[]` Fund PDA
    /// 2. `[writable]` InsuranceFundConfig PDA
    /// 3. `[]` Fund vault PDA
    UpdateHourlySnapshot,
    
    /// Set ADL in progress status (CPI from Ledger)
    /// 
    /// Accounts:
    /// 0. `[signer]` Caller program (Ledger)
    /// 1. `[writable]` InsuranceFundConfig PDA
    SetADLInProgress(SetADLInProgressArgs),
    
    /// Check ADL trigger conditions (view)
    /// 
    /// Accounts:
    /// 0. `[]` Fund PDA
    /// 1. `[]` InsuranceFundConfig PDA
    /// 2. `[]` Fund vault PDA
    CheckADLTrigger(CheckADLTriggerArgs),
    
    /// Redeem shares from Insurance Fund (with special rules)
    /// 
    /// Special rules for Insurance Fund LP redemption:
    /// 1. ADL in progress: redemption is paused
    /// 2. Withdrawal delay: must wait for configured delay after request
    /// 
    /// Accounts:
    /// 0. `[signer]` LP investor
    /// 1. `[writable]` Fund PDA (Insurance Fund)
    /// 2. `[]` InsuranceFundConfig PDA
    /// 3. `[writable]` Fund vault PDA
    /// 4. `[writable]` LP's USDC account
    /// 5. `[writable]` LP Position PDA
    /// 6. `[writable]` LP's share token account
    /// 7. `[writable]` Share mint PDA
    /// 8. `[]` Token Program
    RedeemFromInsuranceFund(RedeemFromInsuranceFundArgs),
    
    // === Square Platform Operations (90-99) ===
    
    /// Process a Square platform payment
    /// 
    /// Records payment on-chain, transfers creator share to their Vault,
    /// and platform share to Square Fund.
    /// 
    /// Supports: knowledge purchases, subscriptions, live donations
    /// 
    /// Accounts:
    /// 0. `[signer]` Payer (user)
    /// 1. `[writable]` SquarePaymentRecord PDA
    /// 2. `[writable]` Payer's Vault (source)
    /// 3. `[writable]` Creator's Vault (destination for creator share)
    /// 4. `[writable]` Square Fund vault (destination for platform share)
    /// 5. `[]` Vault Program
    /// 6. `[]` Token Program
    /// 7. `[]` System Program
    SquarePayment(SquarePaymentArgs),
}

// === Argument Structs ===

/// Arguments for Initialize instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct InitializeArgs {
    /// Vault Program ID
    pub vault_program: Pubkey,
    /// Ledger Program ID
    pub ledger_program: Pubkey,
}

/// Arguments for CreateFund instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CreateFundArgs {
    /// Fund name (max 32 characters)
    pub name: String,
    /// Management fee in basis points (e.g., 200 = 2%)
    pub management_fee_bps: u32,
    /// Performance fee in basis points (e.g., 2000 = 20%)
    pub performance_fee_bps: u32,
    /// Use High Water Mark for performance fee
    pub use_high_water_mark: bool,
    /// Fee collection interval in seconds (0 = default 1 day)
    pub fee_collection_interval: i64,
}

/// Arguments for UpdateFund instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UpdateFundArgs {
    /// New fee configuration (optional)
    pub fee_config: Option<FeeConfig>,
}

/// Arguments for SetFundOpen instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SetFundOpenArgs {
    /// Whether the fund is open for deposits
    pub is_open: bool,
}

/// Arguments for SetFundPaused instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SetFundPausedArgs {
    /// Whether the fund is paused
    pub is_paused: bool,
}

/// Arguments for DepositToFund instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct DepositToFundArgs {
    /// Amount to deposit (in USDC, 6 decimals)
    pub amount: u64,
}

/// Arguments for RedeemFromFund instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RedeemFromFundArgs {
    /// Number of shares to redeem
    pub shares: u64,
}

/// Arguments for TradeFund instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TradeFundArgs {
    /// Market index
    pub market_index: u8,
    /// Side (0 = Long, 1 = Short)
    pub side: u8,
    /// Position size (in e6)
    pub size_e6: u64,
    /// Entry price (in e6)
    pub price_e6: u64,
    /// Leverage (1-100)
    pub leverage: u8,
    /// Maximum slippage in basis points
    pub max_slippage_bps: u32,
}

/// Arguments for CloseFundPosition instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CloseFundPositionArgs {
    /// Market index
    pub market_index: u8,
    /// Close size (in e6, 0 = close all)
    pub size_e6: u64,
    /// Exit price (in e6)
    pub price_e6: u64,
}

/// Arguments for UpdateAuthority instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UpdateAuthorityArgs {
    /// New authority public key
    pub new_authority: Pubkey,
}

/// Arguments for SetProgramPaused instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SetProgramPausedArgs {
    /// Whether the program is paused
    pub is_paused: bool,
}

/// Arguments for RecordPnL instruction (CPI)
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RecordPnLArgs {
    /// Realized PnL amount (can be negative)
    pub pnl_e6: i64,
}

// === Insurance Fund Argument Structs ===

/// Arguments for InitializeInsuranceFund instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct InitializeInsuranceFundArgs {
    /// ADL balance insufficiency trigger threshold (e6)
    pub adl_trigger_threshold_e6: i64,
    /// LP redemption delay in seconds
    pub withdrawal_delay_secs: i64,
    /// Authorized caller (Ledger Program)
    pub authorized_caller: Pubkey,
}

/// Arguments for AddLiquidationIncome instruction (CPI)
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AddLiquidationIncomeArgs {
    /// Liquidation income amount (e6)
    pub amount_e6: i64,
}

/// Arguments for AddADLProfit instruction (CPI)
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AddADLProfitArgs {
    /// ADL profit amount (e6)
    pub amount_e6: i64,
}

/// Arguments for CoverShortfall instruction (CPI)
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CoverShortfallArgs {
    /// Shortfall amount to cover (e6)
    pub shortfall_e6: i64,
}

/// Arguments for SetADLInProgress instruction (CPI)
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SetADLInProgressArgs {
    /// Whether ADL is in progress
    pub in_progress: bool,
}

/// Arguments for CheckADLTrigger instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CheckADLTriggerArgs {
    /// Shortfall to check against (e6), 0 for no bankruptcy check
    pub shortfall_e6: i64,
}

/// Arguments for RedeemFromInsuranceFund instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RedeemFromInsuranceFundArgs {
    /// Number of shares to redeem
    pub shares: u64,
}

// === Square Platform Argument Structs ===

/// Arguments for SquarePayment instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SquarePaymentArgs {
    /// Creator address (content owner)
    pub creator: Pubkey,
    /// Content ID (unique identifier for the content)
    pub content_id: u64,
    /// Payment type: 0 = KnowledgePurchase, 1 = Subscription, 2 = LiveDonation
    pub payment_type: u8,
    /// Total payment amount (e6)
    pub amount_e6: i64,
    /// Creator share in basis points (e.g., 9000 = 90%)
    pub creator_share_bps: u16,
    /// Subscription period (number of months, 0 for non-subscription)
    pub subscription_period: u8,
    /// Optional memo (max 32 bytes)
    pub memo: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_serialization() {
        // Test CreateFund serialization
        let args = CreateFundArgs {
            name: "Test Fund".to_string(),
            management_fee_bps: 200,
            performance_fee_bps: 2000,
            use_high_water_mark: true,
            fee_collection_interval: 86400,
        };
        let ix = FundInstruction::CreateFund(args);
        let serialized = ix.try_to_vec().unwrap();
        assert!(!serialized.is_empty());
        
        // Test deserialization
        let deserialized: FundInstruction = BorshDeserialize::try_from_slice(&serialized).unwrap();
        match deserialized {
            FundInstruction::CreateFund(a) => {
                assert_eq!(a.name, "Test Fund");
                assert_eq!(a.management_fee_bps, 200);
            }
            _ => panic!("Wrong instruction type"),
        }
    }

    #[test]
    fn test_deposit_instruction() {
        let args = DepositToFundArgs { amount: 1_000_000 };
        let ix = FundInstruction::DepositToFund(args);
        let serialized = ix.try_to_vec().unwrap();
        
        let deserialized: FundInstruction = BorshDeserialize::try_from_slice(&serialized).unwrap();
        match deserialized {
            FundInstruction::DepositToFund(a) => {
                assert_eq!(a.amount, 1_000_000);
            }
            _ => panic!("Wrong instruction type"),
        }
    }

    #[test]
    fn test_redeem_instruction() {
        let args = RedeemFromFundArgs { shares: 500_000 };
        let ix = FundInstruction::RedeemFromFund(args);
        let serialized = ix.try_to_vec().unwrap();
        
        let deserialized: FundInstruction = BorshDeserialize::try_from_slice(&serialized).unwrap();
        match deserialized {
            FundInstruction::RedeemFromFund(a) => {
                assert_eq!(a.shares, 500_000);
            }
            _ => panic!("Wrong instruction type"),
        }
    }
}

