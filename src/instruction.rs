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
    
    /// Add trading fee income to Insurance Fund (CPI from Ledger)
    /// V1 简化方案: 交易手续费直接转入保险基金
    /// 
    /// Accounts:
    /// 0. `[signer]` Caller program (Ledger)
    /// 1. `[writable]` Fund PDA (Insurance Fund)
    /// 2. `[writable]` InsuranceFundConfig PDA
    /// 3. `[writable]` Vault Token Account (source of fees)
    /// 4. `[writable]` Insurance Fund Vault (destination)
    /// 5. `[]` Token Program
    AddTradingFee(AddTradingFeeArgs),
    
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
    
    // === Referral Operations (100-119) ===
    
    /// Initialize Referral configuration
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority (admin)
    /// 1. `[writable]` ReferralConfig PDA
    /// 2. `[]` Vault Program
    /// 3. `[]` System Program
    InitializeReferral(InitializeReferralArgs),
    
    /// Create a referral link
    /// 
    /// Accounts:
    /// 0. `[signer]` Referrer
    /// 1. `[writable]` ReferralLink PDA
    /// 2. `[writable]` ReferralConfig PDA
    /// 3. `[]` System Program
    CreateReferralLink(CreateReferralLinkArgs),
    
    /// Bind referral relationship (new user registration)
    /// 
    /// Accounts:
    /// 0. `[signer]` Referee (new user)
    /// 1. `[writable]` ReferralBinding PDA
    /// 2. `[]` ReferralLink
    /// 3. `[writable]` ReferralLink (update stats)
    /// 4. `[writable]` ReferralConfig (update stats)
    /// 5. `[]` System Program
    BindReferral,
    
    /// Record a referral trade (CPI from Ledger)
    /// 
    /// Records the trade and calculates rewards.
    /// Actual token transfers happen in Ledger/Vault.
    /// 
    /// Accounts:
    /// 0. `[signer]` Caller program (Ledger)
    /// 1. `[]` ReferralConfig
    /// 2. `[writable]` ReferralBinding
    /// 3. `[writable]` ReferralLink
    RecordReferralTrade(RecordReferralTradeArgs),
    
    /// Update Referral configuration
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority
    /// 1. `[writable]` ReferralConfig PDA
    UpdateReferralConfig(UpdateReferralConfigArgs),
    
    /// Deactivate a referral link
    /// 
    /// Accounts:
    /// 0. `[signer]` Referrer (link owner)
    /// 1. `[writable]` ReferralLink PDA
    DeactivateReferralLink,
    
    /// Set custom rates for a referral link
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority (admin only)
    /// 1. `[writable]` ReferralLink PDA
    SetCustomReferralRates(SetCustomReferralRatesArgs),
    
    // =========================================================================
    // Prediction Market Fee Operations (120-139)
    // =========================================================================
    
    // =========================================================================
    // Relayer Instructions (200-249) - Admin/Relayer 代替用户签名
    // =========================================================================
    
    /// Relayer 版本的 DepositToFund
    /// 
    /// Accounts:
    /// 0. `[signer]` Admin/Relayer
    /// 1. `[writable]` Fund PDA
    /// 2. `[writable]` Fund vault PDA
    /// 3. `[writable]` User's Vault Account (Vault Program)
    /// 4. `[writable]` LP Position PDA
    /// 5. `[writable]` LP's share token account
    /// 6. `[writable]` Share mint PDA
    /// 7. `[]` VaultConfig
    /// 8. `[]` Vault Program
    /// 9. `[]` Token Program
    /// 10. `[]` System Program
    RelayerDepositToFund(RelayerDepositToFundArgs),
    
    /// Relayer 版本的 RedeemFromFund
    RelayerRedeemFromFund(RelayerRedeemFromFundArgs),
    
    /// Relayer 版本的 RedeemFromInsuranceFund
    RelayerRedeemFromInsuranceFund(RelayerRedeemFromInsuranceFundArgs),
    
    /// Relayer 版本的 SquarePayment
    RelayerSquarePayment(RelayerSquarePaymentArgs),
    
    /// Relayer 版本的 BindReferral
    RelayerBindReferral(RelayerBindReferralArgs),
    
    // =========================================================================
    // Relayer Management Instructions (250-259)
    // =========================================================================
    
    /// 添加授权 Relayer (Admin only)
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority (admin)
    /// 1. `[writable]` FundConfig PDA
    AddRelayer(AddRelayerArgs),
    
    /// 移除 Relayer (Admin only)
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority (admin)
    /// 1. `[writable]` FundConfig PDA
    RemoveRelayer(RemoveRelayerArgs),
    
    /// 更新 Relayer 限额配置 (Admin only)
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority (admin)
    /// 1. `[writable]` FundConfig PDA
    UpdateRelayerLimits(UpdateRelayerLimitsArgs),

    /// 初始化预测市场手续费配置
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority (admin)
    /// 1. `[writable]` PredictionMarketFeeConfig PDA
    /// 2. `[writable]` Prediction Market Fee Vault PDA (Token Account)
    /// 3. `[]` USDC Mint
    /// 4. `[]` Prediction Market Program (authorized caller)
    /// 5. `[]` Token Program
    /// 6. `[]` System Program
    InitializePredictionMarketFeeConfig(InitializePredictionMarketFeeConfigArgs),
    
    /// 收取预测市场铸造手续费 (CPI from Prediction Market Program)
    /// 
    /// Accounts:
    /// 0. `[signer]` Caller Program
    /// 1. `[writable]` PredictionMarketFeeConfig
    /// 2. `[writable]` Prediction Market Fee Vault
    /// 3. `[writable]` Source Token Account (用户的 USDC)
    /// 4. `[]` Token Program
    CollectPredictionMarketMintingFee(CollectPredictionMarketMintingFeeArgs),
    
    /// 收取预测市场赎回手续费 (CPI from Prediction Market Program)
    /// 
    /// Accounts:
    /// 0. `[signer]` Caller Program
    /// 1. `[writable]` PredictionMarketFeeConfig
    /// 2. `[writable]` Prediction Market Fee Vault
    /// 3. `[writable]` Source Token Account
    /// 4. `[]` Token Program
    CollectPredictionMarketRedemptionFee(CollectPredictionMarketRedemptionFeeArgs),
    
    /// 收取预测市场交易手续费 (CPI from Prediction Market Program)
    /// 
    /// Accounts:
    /// 0. `[signer]` Caller Program
    /// 1. `[writable]` PredictionMarketFeeConfig
    /// 2. `[writable]` Prediction Market Fee Vault
    /// 3. `[writable]` Source Token Account
    /// 4. `[]` Token Program
    CollectPredictionMarketTradingFee(CollectPredictionMarketTradingFeeArgs),
    
    /// 发放预测市场做市商奖励 (Admin or CPI)
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority or Caller
    /// 1. `[writable]` PredictionMarketFeeConfig
    /// 2. `[writable]` Prediction Market Fee Vault
    /// 3. `[writable]` Maker's Token Account
    /// 4. `[]` Token Program
    DistributePredictionMarketMakerReward(DistributePredictionMarketMakerRewardArgs),
    
    /// 发放预测市场创建者分成 (CPI)
    /// 
    /// Accounts:
    /// 0. `[signer]` Caller Program
    /// 1. `[writable]` PredictionMarketFeeConfig
    /// 2. `[writable]` Prediction Market Fee Vault
    /// 3. `[writable]` Creator's Token Account
    /// 4. `[]` Token Program
    DistributePredictionMarketCreatorReward(DistributePredictionMarketCreatorRewardArgs),
    
    /// 更新预测市场手续费配置
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority
    /// 1. `[writable]` PredictionMarketFeeConfig
    UpdatePredictionMarketFeeConfig(UpdatePredictionMarketFeeConfigArgs),
    
    /// 设置预测市场手续费暂停状态
    /// 
    /// Accounts:
    /// 0. `[signer]` Authority
    /// 1. `[writable]` PredictionMarketFeeConfig
    SetPredictionMarketFeePaused(SetPredictionMarketFeePausedArgs),
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

/// Arguments for AddTradingFee instruction (CPI)
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AddTradingFeeArgs {
    /// Trading fee amount (e6)
    pub fee_e6: i64,
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

// === Referral Argument Structs ===

/// Arguments for InitializeReferral instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct InitializeReferralArgs {
    /// Base referrer share in basis points (e.g., 2000 = 20%)
    pub referrer_share_bps: u16,
    /// Base referee discount in basis points (e.g., 1000 = 10%)
    pub referee_discount_bps: u16,
}

/// Arguments for CreateReferralLink instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CreateReferralLinkArgs {
    /// Referral code (6-12 characters)
    pub code: Vec<u8>,
}

/// Arguments for RecordReferralTrade instruction (CPI)
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RecordReferralTradeArgs {
    /// Trade fee amount (e6)
    pub trade_fee_e6: i64,
    /// Trade volume (e6)
    pub trade_volume_e6: i64,
    /// Referrer VIP level
    pub referrer_vip_level: u8,
    /// Referee VIP level
    pub referee_vip_level: u8,
}

/// Arguments for UpdateReferralConfig instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UpdateReferralConfigArgs {
    /// New referrer share in basis points (None = no change)
    pub referrer_share_bps: Option<u16>,
    /// New referee discount in basis points (None = no change)
    pub referee_discount_bps: Option<u16>,
    /// New referrer VIP bonus array (None = no change)
    pub referrer_vip_bonus_bps: Option<[u16; 6]>,
    /// New referee VIP bonus array (None = no change)
    pub referee_vip_bonus_bps: Option<[u16; 6]>,
    /// New minimum settlement amount (None = no change)
    pub min_settlement_amount_e6: Option<i64>,
    /// Pause/unpause (None = no change)
    pub is_paused: Option<bool>,
}

/// Arguments for SetCustomReferralRates instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SetCustomReferralRatesArgs {
    /// Custom referrer share in basis points (0 = use global)
    pub custom_referrer_share_bps: u16,
    /// Custom referee discount in basis points (0 = use global)
    pub custom_referee_discount_bps: u16,
}

// === Prediction Market Fee Argument Structs ===

/// Arguments for InitializePredictionMarketFeeConfig instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct InitializePredictionMarketFeeConfigArgs {
    /// Prediction market minting fee in basis points (default 10 = 0.1%)
    pub prediction_market_minting_fee_bps: u16,
    /// Prediction market redemption fee in basis points
    pub prediction_market_redemption_fee_bps: u16,
    /// Prediction market taker trading fee in basis points
    pub prediction_market_trading_fee_taker_bps: u16,
    /// Prediction market maker trading fee in basis points (usually 0)
    pub prediction_market_trading_fee_maker_bps: u16,
    /// Prediction market protocol share of fees in basis points
    pub prediction_market_protocol_share_bps: u16,
    /// Prediction market maker reward share in basis points
    pub prediction_market_maker_reward_share_bps: u16,
    /// Prediction market creator share in basis points
    pub prediction_market_creator_share_bps: u16,
}

/// Arguments for CollectPredictionMarketMintingFee instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CollectPredictionMarketMintingFeeArgs {
    /// Prediction market minting amount (e6) - fee calculated based on this
    pub prediction_market_minting_amount_e6: i64,
}

/// Arguments for CollectPredictionMarketRedemptionFee instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CollectPredictionMarketRedemptionFeeArgs {
    /// Prediction market redemption amount (e6) - fee calculated based on this
    pub prediction_market_redemption_amount_e6: i64,
}

/// Arguments for CollectPredictionMarketTradingFee instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CollectPredictionMarketTradingFeeArgs {
    /// Prediction market trade volume (e6) - fee calculated based on this
    pub prediction_market_trade_volume_e6: i64,
    /// Is this a taker fee?
    pub is_taker: bool,
}

/// Arguments for DistributePredictionMarketMakerReward instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct DistributePredictionMarketMakerRewardArgs {
    /// Prediction market maker reward amount (e6)
    pub prediction_market_maker_reward_e6: i64,
}

/// Arguments for DistributePredictionMarketCreatorReward instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct DistributePredictionMarketCreatorRewardArgs {
    /// Prediction market creator reward amount (e6)
    pub prediction_market_creator_reward_e6: i64,
    /// Prediction market ID (for tracking)
    pub prediction_market_id: u64,
}

/// Arguments for UpdatePredictionMarketFeeConfig instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UpdatePredictionMarketFeeConfigArgs {
    /// New prediction market minting fee (None = no change)
    pub prediction_market_minting_fee_bps: Option<u16>,
    /// New prediction market redemption fee (None = no change)
    pub prediction_market_redemption_fee_bps: Option<u16>,
    /// New prediction market taker trading fee (None = no change)
    pub prediction_market_trading_fee_taker_bps: Option<u16>,
    /// New prediction market maker trading fee (None = no change)
    pub prediction_market_trading_fee_maker_bps: Option<u16>,
    /// New prediction market protocol share (None = no change)
    pub prediction_market_protocol_share_bps: Option<u16>,
    /// New prediction market maker reward share (None = no change)
    pub prediction_market_maker_reward_share_bps: Option<u16>,
    /// New prediction market creator share (None = no change)
    pub prediction_market_creator_share_bps: Option<u16>,
}

/// Arguments for SetPredictionMarketFeePaused instruction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SetPredictionMarketFeePausedArgs {
    /// Prediction market fee paused state
    pub prediction_market_fee_paused: bool,
}

// ============================================================================
// Relayer Instructions (200-249) - Admin/Relayer 代替用户签名
// ============================================================================

/// Relayer 版本的 DepositToFund
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RelayerDepositToFundArgs {
    /// 用户钱包地址
    pub user_wallet: Pubkey,
    /// Amount to deposit (in USDC, 6 decimals)
    pub amount: u64,
}

/// Relayer 版本的 RedeemFromFund
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RelayerRedeemFromFundArgs {
    /// 用户钱包地址
    pub user_wallet: Pubkey,
    /// Number of shares to redeem
    pub shares: u64,
}

/// Relayer 版本的 RedeemFromInsuranceFund
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RelayerRedeemFromInsuranceFundArgs {
    /// 用户钱包地址
    pub user_wallet: Pubkey,
    /// Number of shares to redeem
    pub shares: u64,
}

/// Relayer 版本的 SquarePayment
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RelayerSquarePaymentArgs {
    /// 付款用户钱包地址
    pub payer_wallet: Pubkey,
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

/// Relayer 版本的 BindReferral
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RelayerBindReferralArgs {
    /// 新用户钱包地址
    pub user_wallet: Pubkey,
    /// Referral link address
    pub referral_link: Pubkey,
}

// ============================================================================
// Relayer Management Instructions (250-259)
// ============================================================================

/// 添加授权 Relayer
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AddRelayerArgs {
    /// 新 Relayer 公钥
    pub relayer: Pubkey,
}

/// 移除 Relayer
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RemoveRelayerArgs {
    /// 要移除的 Relayer 公钥
    pub relayer: Pubkey,
}

/// 更新 Relayer 限额配置
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UpdateRelayerLimitsArgs {
    /// 单笔交易限额 (e6), 0 = 无限制
    pub single_tx_limit_e6: Option<i64>,
    /// 每日限额 (e6), 0 = 无限制
    pub daily_limit_e6: Option<i64>,
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

