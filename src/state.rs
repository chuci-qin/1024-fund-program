//! Fund Program State Definitions
//!
//! Defines all account structures for the Fund Program.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::utils::{
    calculate_management_fee, calculate_nav_e6, calculate_performance_fee,
    safe_add_i64, INITIAL_NAV_E6, MAX_FUND_NAME_LEN,
};
use solana_program::program_error::ProgramError;

// === Discriminators ===

/// Discriminator for FundConfig account
pub const FUND_CONFIG_DISCRIMINATOR: u64 = 0x46554E445F434F4E; // "FUND_CON"

/// Discriminator for Fund account
pub const FUND_DISCRIMINATOR: u64 = 0x46554E445F46554E; // "FUND_FUN"

/// Discriminator for LPPosition account
pub const LP_POSITION_DISCRIMINATOR: u64 = 0x4C505F504F534954; // "LP_POSIT"

/// Discriminator for InsuranceFundConfig account
pub const INSURANCE_FUND_CONFIG_DISCRIMINATOR: u64 = 0x494E5355525F4346; // "INSUR_CF"

/// Discriminator for SquarePaymentRecord account
pub const SQUARE_PAYMENT_RECORD_DISCRIMINATOR: u64 = 0x5351555F50415952; // "SQU_PAYR"

/// Discriminator for ReferralConfig account
pub const REFERRAL_CONFIG_DISCRIMINATOR: u64 = 0x5245465F434F4E46; // "REF_CONF"

/// Discriminator for ReferralLink account
pub const REFERRAL_LINK_DISCRIMINATOR: u64 = 0x5245465F4C494E4B; // "REF_LINK"

/// Discriminator for ReferralBinding account
pub const REFERRAL_BINDING_DISCRIMINATOR: u64 = 0x5245465F42494E44; // "REF_BIND"

// === PDA Seeds ===

/// Seed prefix for FundConfig PDA
pub const FUND_CONFIG_SEED: &[u8] = b"fund_config";

/// Seed prefix for Fund PDA
pub const FUND_SEED: &[u8] = b"fund";

/// Seed prefix for Fund vault PDA
pub const FUND_VAULT_SEED: &[u8] = b"fund_vault";

/// Seed prefix for Share mint PDA
pub const SHARE_MINT_SEED: &[u8] = b"share_mint";

/// Seed prefix for LP position PDA
pub const LP_POSITION_SEED: &[u8] = b"lp_position";

/// Seed prefix for InsuranceFundConfig PDA
pub const INSURANCE_FUND_CONFIG_SEED: &[u8] = b"insurance_fund_config";

/// Seed prefix for SquarePaymentRecord PDA
pub const SQUARE_PAYMENT_RECORD_SEED: &[u8] = b"square_payment";

/// Seed prefix for ReferralConfig PDA
pub const REFERRAL_CONFIG_SEED: &[u8] = b"referral_config";

/// Seed prefix for ReferralLink PDA
pub const REFERRAL_LINK_SEED: &[u8] = b"referral_link";

/// Seed prefix for ReferralBinding PDA
pub const REFERRAL_BINDING_SEED: &[u8] = b"referral_binding";

// === Fund Config ===

/// Global configuration for the Fund Program
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct FundConfig {
    /// Discriminator for account type
    pub discriminator: u64,
    
    /// Program authority (admin)
    pub authority: Pubkey,
    
    /// Vault Program ID (for CPI)
    pub vault_program: Pubkey,
    
    /// Ledger Program ID (for CPI)
    pub ledger_program: Pubkey,
    
    /// Total number of funds created
    pub total_funds: u64,
    
    /// Total number of active funds
    pub active_funds: u64,
    
    /// Total value locked across all funds (e6)
    pub total_tvl_e6: i64,
    
    /// Is the program paused?
    pub is_paused: bool,
    
    /// PDA bump
    pub bump: u8,
    
    /// Reserved for future use
    pub reserved: [u8; 64],
}

impl FundConfig {
    /// Account size in bytes
    pub const SIZE: usize = 8  // discriminator
        + 32  // authority
        + 32  // vault_program
        + 32  // ledger_program
        + 8   // total_funds
        + 8   // active_funds
        + 8   // total_tvl_e6
        + 1   // is_paused
        + 1   // bump
        + 64; // reserved
    
    /// Create a new FundConfig
    pub fn new(authority: Pubkey, vault_program: Pubkey, ledger_program: Pubkey, bump: u8) -> Self {
        Self {
            discriminator: FUND_CONFIG_DISCRIMINATOR,
            authority,
            vault_program,
            ledger_program,
            total_funds: 0,
            active_funds: 0,
            total_tvl_e6: 0,
            is_paused: false,
            bump,
            reserved: [0u8; 64],
        }
    }
    
    /// PDA seeds for FundConfig
    pub fn seeds() -> Vec<Vec<u8>> {
        vec![FUND_CONFIG_SEED.to_vec()]
    }
}

// === Fee Config ===

/// Fee configuration for a fund
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, Default)]
pub struct FeeConfig {
    /// Management fee in basis points (e.g., 200 = 2%)
    pub management_fee_bps: u32,
    
    /// Performance fee in basis points (e.g., 2000 = 20%)
    pub performance_fee_bps: u32,
    
    /// Use High Water Mark for performance fee?
    pub use_high_water_mark: bool,
    
    /// Minimum interval between fee collections (seconds)
    pub fee_collection_interval: i64,
}

impl FeeConfig {
    /// Size in bytes
    pub const SIZE: usize = 4  // management_fee_bps
        + 4  // performance_fee_bps
        + 1  // use_high_water_mark
        + 8; // fee_collection_interval
    
    /// Default fee collection interval (1 day)
    pub const DEFAULT_COLLECTION_INTERVAL: i64 = 24 * 60 * 60;
    
    /// Create a new FeeConfig with default values
    pub fn new(management_fee_bps: u32, performance_fee_bps: u32) -> Self {
        Self {
            management_fee_bps,
            performance_fee_bps,
            use_high_water_mark: true,
            fee_collection_interval: Self::DEFAULT_COLLECTION_INTERVAL,
        }
    }
}

// === Fund Stats ===

/// Statistics for a fund
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Default)]
pub struct FundStats {
    /// Total USDC deposited (e6)
    pub total_deposits_e6: i64,
    
    /// Total USDC withdrawn (e6)
    pub total_withdrawals_e6: i64,
    
    /// Current NAV per share (e6, 1.0 = 1_000_000)
    pub current_nav_e6: i64,
    
    /// High Water Mark for performance fee (e6)
    pub high_water_mark_e6: i64,
    
    /// Total management fees collected (e6)
    pub total_management_fee_e6: i64,
    
    /// Total performance fees collected (e6)
    pub total_performance_fee_e6: i64,
    
    /// Total shares outstanding
    pub total_shares: u64,
    
    /// Last fee collection timestamp
    pub last_fee_collection_ts: i64,
    
    /// Total realized PnL (e6)
    pub total_realized_pnl_e6: i64,
    
    /// Number of LP investors
    pub lp_count: u32,
}

impl FundStats {
    /// Size in bytes
    pub const SIZE: usize = 8  // total_deposits_e6
        + 8  // total_withdrawals_e6
        + 8  // current_nav_e6
        + 8  // high_water_mark_e6
        + 8  // total_management_fee_e6
        + 8  // total_performance_fee_e6
        + 8  // total_shares
        + 8  // last_fee_collection_ts
        + 8  // total_realized_pnl_e6
        + 4; // lp_count
    
    /// Create new FundStats with initial values
    pub fn new(created_at: i64) -> Self {
        Self {
            total_deposits_e6: 0,
            total_withdrawals_e6: 0,
            current_nav_e6: INITIAL_NAV_E6,
            high_water_mark_e6: INITIAL_NAV_E6,
            total_management_fee_e6: 0,
            total_performance_fee_e6: 0,
            total_shares: 0,
            last_fee_collection_ts: created_at,
            total_realized_pnl_e6: 0,
            lp_count: 0,
        }
    }
    
    /// Get total value of the fund (e6)
    pub fn total_value_e6(&self) -> i64 {
        // Total value = deposits - withdrawals + realized PnL - fees
        self.total_deposits_e6
            .saturating_sub(self.total_withdrawals_e6)
            .saturating_add(self.total_realized_pnl_e6)
            .saturating_sub(self.total_management_fee_e6)
            .saturating_sub(self.total_performance_fee_e6)
    }
    
    /// Update NAV based on current total value
    pub fn update_nav(&mut self) -> Result<(), ProgramError> {
        self.current_nav_e6 = calculate_nav_e6(self.total_value_e6(), self.total_shares)?;
        Ok(())
    }
    
    /// Update High Water Mark if current NAV exceeds it
    pub fn update_hwm(&mut self) {
        if self.current_nav_e6 > self.high_water_mark_e6 {
            self.high_water_mark_e6 = self.current_nav_e6;
        }
    }
    
    /// Calculate and collect fees
    pub fn collect_fees(
        &mut self,
        _current_ts: i64,
    ) -> Result<(i64, i64), ProgramError> {
        // This method should be called with FeeConfig, simplified here
        // Returns (management_fee, performance_fee)
        Ok((0, 0))
    }
}

// === Fund ===

/// A single fund managed by a fund manager
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Fund {
    /// Discriminator for account type
    pub discriminator: u64,
    
    /// Fund manager (owner)
    pub manager: Pubkey,
    
    /// Fund name (max 32 bytes)
    pub name: [u8; MAX_FUND_NAME_LEN],
    
    /// PDA bump
    pub bump: u8,
    
    /// Fund's USDC vault account
    pub fund_vault: Pubkey,
    
    /// Share token mint
    pub share_mint: Pubkey,
    
    /// Fee configuration
    pub fee_config: FeeConfig,
    
    /// Fund statistics
    pub stats: FundStats,
    
    /// Is the fund open for deposits?
    pub is_open: bool,
    
    /// Is the fund paused?
    pub is_paused: bool,
    
    /// Fund creation timestamp
    pub created_at: i64,
    
    /// Last update timestamp
    pub last_update_ts: i64,
    
    /// Fund index (unique identifier)
    pub fund_index: u64,
    
    /// Reserved for future use
    pub reserved: [u8; 64],
}

impl Fund {
    /// Account size in bytes
    pub const SIZE: usize = 8  // discriminator
        + 32  // manager
        + MAX_FUND_NAME_LEN  // name
        + 1   // bump
        + 32  // fund_vault
        + 32  // share_mint
        + FeeConfig::SIZE  // fee_config
        + FundStats::SIZE  // stats
        + 1   // is_open
        + 1   // is_paused
        + 8   // created_at
        + 8   // last_update_ts
        + 8   // fund_index
        + 64; // reserved
    
    /// Create a new Fund
    pub fn new(
        manager: Pubkey,
        name: &str,
        bump: u8,
        fund_vault: Pubkey,
        share_mint: Pubkey,
        fee_config: FeeConfig,
        fund_index: u64,
        created_at: i64,
    ) -> Self {
        let mut name_bytes = [0u8; MAX_FUND_NAME_LEN];
        let name_len = name.len().min(MAX_FUND_NAME_LEN);
        name_bytes[..name_len].copy_from_slice(&name.as_bytes()[..name_len]);
        
        Self {
            discriminator: FUND_DISCRIMINATOR,
            manager,
            name: name_bytes,
            bump,
            fund_vault,
            share_mint,
            fee_config,
            stats: FundStats::new(created_at),
            is_open: true,
            is_paused: false,
            created_at,
            last_update_ts: created_at,
            fund_index,
            reserved: [0u8; 64],
        }
    }
    
    /// Get fund name as string
    pub fn name_str(&self) -> String {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(self.name.len());
        String::from_utf8_lossy(&self.name[..end]).to_string()
    }
    
    /// PDA seeds for Fund
    pub fn seeds(manager: &Pubkey, fund_index: u64) -> Vec<Vec<u8>> {
        vec![
            FUND_SEED.to_vec(),
            manager.to_bytes().to_vec(),
            fund_index.to_le_bytes().to_vec(),
        ]
    }
    
    /// PDA seeds for Fund vault
    pub fn vault_seeds(fund: &Pubkey) -> Vec<Vec<u8>> {
        vec![
            FUND_VAULT_SEED.to_vec(),
            fund.to_bytes().to_vec(),
        ]
    }
    
    /// PDA seeds for Share mint
    pub fn share_mint_seeds(fund: &Pubkey) -> Vec<Vec<u8>> {
        vec![
            SHARE_MINT_SEED.to_vec(),
            fund.to_bytes().to_vec(),
        ]
    }
    
    /// Check if this fund is the correct manager
    pub fn is_manager(&self, signer: &Pubkey) -> bool {
        self.manager == *signer
    }
    
    /// Check if deposits are allowed
    pub fn can_deposit(&self) -> bool {
        self.is_open && !self.is_paused
    }
    
    /// Check if withdrawals are allowed
    pub fn can_withdraw(&self) -> bool {
        !self.is_paused
    }
    
    /// Record a deposit
    pub fn record_deposit(&mut self, amount_e6: i64, shares: u64) -> Result<(), ProgramError> {
        self.stats.total_deposits_e6 = safe_add_i64(self.stats.total_deposits_e6, amount_e6)?;
        self.stats.total_shares = self.stats.total_shares.saturating_add(shares);
        self.stats.update_nav()?;
        Ok(())
    }
    
    /// Record a withdrawal
    pub fn record_withdrawal(&mut self, amount_e6: i64, shares: u64) -> Result<(), ProgramError> {
        self.stats.total_withdrawals_e6 = safe_add_i64(self.stats.total_withdrawals_e6, amount_e6)?;
        self.stats.total_shares = self.stats.total_shares.saturating_sub(shares);
        self.stats.update_nav()?;
        Ok(())
    }
    
    /// Record realized PnL
    pub fn record_pnl(&mut self, pnl_e6: i64) -> Result<(), ProgramError> {
        self.stats.total_realized_pnl_e6 = safe_add_i64(self.stats.total_realized_pnl_e6, pnl_e6)?;
        self.stats.update_nav()?;
        self.stats.update_hwm();
        Ok(())
    }
    
    /// Calculate and record fees
    pub fn calculate_fees(
        &self,
        current_ts: i64,
    ) -> Result<(i64, i64), ProgramError> {
        let time_elapsed = current_ts - self.stats.last_fee_collection_ts;
        if time_elapsed <= 0 {
            return Ok((0, 0));
        }
        
        let total_value = self.stats.total_value_e6();
        
        // Calculate management fee
        let mgmt_fee = calculate_management_fee(
            total_value,
            self.fee_config.management_fee_bps,
            time_elapsed,
        )?;
        
        // Calculate performance fee
        let perf_fee = if self.fee_config.use_high_water_mark {
            calculate_performance_fee(
                self.stats.current_nav_e6,
                self.stats.high_water_mark_e6,
                total_value,
                self.fee_config.performance_fee_bps,
            )?
        } else {
            0
        };
        
        Ok((mgmt_fee, perf_fee))
    }
    
    /// Collect fees (update state)
    pub fn collect_fees(&mut self, mgmt_fee: i64, perf_fee: i64, current_ts: i64) -> Result<(), ProgramError> {
        self.stats.total_management_fee_e6 = safe_add_i64(self.stats.total_management_fee_e6, mgmt_fee)?;
        self.stats.total_performance_fee_e6 = safe_add_i64(self.stats.total_performance_fee_e6, perf_fee)?;
        self.stats.last_fee_collection_ts = current_ts;
        
        // Update NAV after fee deduction
        self.stats.update_nav()?;
        
        // Update HWM after performance fee
        self.stats.update_hwm();
        
        Ok(())
    }
}

// === LP Position ===

/// An LP investor's position in a fund
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct LPPosition {
    /// Discriminator for account type
    pub discriminator: u64,
    
    /// Fund this position belongs to
    pub fund: Pubkey,
    
    /// Investor wallet
    pub investor: Pubkey,
    
    /// Number of shares held
    pub shares: u64,
    
    /// NAV at time of deposit (for tracking returns)
    pub deposit_nav_e6: i64,
    
    /// Total amount deposited (e6)
    pub total_deposited_e6: i64,
    
    /// Total amount withdrawn (e6)
    pub total_withdrawn_e6: i64,
    
    /// Timestamp of first deposit
    pub deposited_at: i64,
    
    /// Last update timestamp
    pub last_update_ts: i64,
    
    /// PDA bump
    pub bump: u8,
    
    /// Reserved for future use
    pub reserved: [u8; 32],
}

impl LPPosition {
    /// Account size in bytes
    pub const SIZE: usize = 8  // discriminator
        + 32  // fund
        + 32  // investor
        + 8   // shares
        + 8   // deposit_nav_e6
        + 8   // total_deposited_e6
        + 8   // total_withdrawn_e6
        + 8   // deposited_at
        + 8   // last_update_ts
        + 1   // bump
        + 32; // reserved
    
    /// Create a new LP position
    pub fn new(
        fund: Pubkey,
        investor: Pubkey,
        shares: u64,
        deposit_nav_e6: i64,
        deposited_amount_e6: i64,
        deposited_at: i64,
        bump: u8,
    ) -> Self {
        Self {
            discriminator: LP_POSITION_DISCRIMINATOR,
            fund,
            investor,
            shares,
            deposit_nav_e6,
            total_deposited_e6: deposited_amount_e6,
            total_withdrawn_e6: 0,
            deposited_at,
            last_update_ts: deposited_at,
            bump,
            reserved: [0u8; 32],
        }
    }
    
    /// PDA seeds for LP position
    pub fn seeds(fund: &Pubkey, investor: &Pubkey) -> Vec<Vec<u8>> {
        vec![
            LP_POSITION_SEED.to_vec(),
            fund.to_bytes().to_vec(),
            investor.to_bytes().to_vec(),
        ]
    }
    
    /// Calculate current value of position
    pub fn current_value(&self, current_nav_e6: i64) -> i64 {
        // value = shares * nav / 1e6
        ((self.shares as i128) * (current_nav_e6 as i128) / 1_000_000) as i64
    }
    
    /// Calculate unrealized PnL
    pub fn unrealized_pnl(&self, current_nav_e6: i64) -> i64 {
        let current_value = self.current_value(current_nav_e6);
        let net_invested = self.total_deposited_e6.saturating_sub(self.total_withdrawn_e6);
        current_value.saturating_sub(net_invested)
    }
    
    /// Add shares (deposit)
    pub fn add_shares(
        &mut self,
        shares: u64,
        amount_e6: i64,
        current_nav_e6: i64,
        current_ts: i64,
    ) -> Result<(), ProgramError> {
        self.shares = self.shares.saturating_add(shares);
        self.total_deposited_e6 = safe_add_i64(self.total_deposited_e6, amount_e6)?;
        
        // Update weighted average deposit NAV
        // new_avg_nav = (old_shares * old_nav + new_shares * new_nav) / total_shares
        // Simplified: just update to current NAV for now
        self.deposit_nav_e6 = current_nav_e6;
        self.last_update_ts = current_ts;
        
        Ok(())
    }
    
    /// Remove shares (redeem)
    pub fn remove_shares(
        &mut self,
        shares: u64,
        amount_e6: i64,
        current_ts: i64,
    ) -> Result<(), ProgramError> {
        if shares > self.shares {
            return Err(crate::error::FundError::InsufficientShares.into());
        }
        
        self.shares = self.shares.saturating_sub(shares);
        self.total_withdrawn_e6 = safe_add_i64(self.total_withdrawn_e6, amount_e6)?;
        self.last_update_ts = current_ts;
        
        Ok(())
    }
    
    /// Check if position is empty
    pub fn is_empty(&self) -> bool {
        self.shares == 0
    }
}

// =============================================================================
// Insurance Fund Config
// =============================================================================

/// ADL 触发原因
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ADLTriggerReason {
    /// 不需要触发 ADL
    None = 0,
    /// 穿仓触发 (保险基金无法覆盖)
    Bankruptcy = 1,
    /// 余额不足触发 (低于阈值)
    InsufficientBalance = 2,
    /// 1小时内快速下降触发 (下降超过30%)
    RapidDecline = 3,
}

impl Default for ADLTriggerReason {
    fn default() -> Self {
        ADLTriggerReason::None
    }
}

/// Insurance Fund 专用配置账户
/// 
/// 这是 Insurance Fund 在 Fund Program 中的扩展配置，
/// 与基础 Fund 账户配合使用。
/// 
/// PDA Seeds: ["insurance_fund_config"]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct InsuranceFundConfig {
    /// 账户类型标识符
    pub discriminator: u64,
    
    /// 关联的 Fund 账户地址
    pub fund: Pubkey,
    
    /// PDA bump
    pub bump: u8,
    
    // === 收入统计 ===
    
    /// 累计清算收入 (e6) - 来自强平罚金
    pub total_liquidation_income_e6: i64,
    
    /// 累计 ADL 盈余收入 (e6) - 来自 ADL 执行
    pub total_adl_profit_e6: i64,
    
    // === 支出统计 ===
    
    /// 累计穿仓支出 (e6) - 用于覆盖穿仓
    pub total_shortfall_payout_e6: i64,
    
    // === ADL 配置 ===
    
    /// ADL 余额不足触发阈值 (e6)
    pub adl_trigger_threshold_e6: i64,
    
    /// ADL 触发次数统计
    pub adl_trigger_count: u64,
    
    // === 1小时快照 (用于30%下降触发条件) ===
    
    /// 1小时前的余额 (e6)
    pub balance_1h_ago_e6: i64,
    
    /// 上次快照时间戳
    pub last_snapshot_ts: i64,
    
    // === LP 赎回控制 ===
    
    /// 赎回延迟 (秒) - 提交赎回后需等待的时间
    pub withdrawal_delay_secs: i64,
    
    /// ADL 进行中标志 - ADL 期间暂停 LP 赎回
    pub is_adl_in_progress: bool,
    
    // === 授权调用方 ===
    
    /// 授权调用 AddLiquidationIncome/AddADLProfit/CoverShortfall 的程序
    pub authorized_caller: Pubkey,
    
    /// 最后更新时间戳
    pub last_update_ts: i64,
    
    /// 预留字段 (扩展用)
    pub reserved: [u8; 64],
}

impl InsuranceFundConfig {
    /// 账户大小 (bytes)
    pub const SIZE: usize = 8   // discriminator
        + 32  // fund
        + 1   // bump
        + 8   // total_liquidation_income_e6
        + 8   // total_adl_profit_e6
        + 8   // total_shortfall_payout_e6
        + 8   // adl_trigger_threshold_e6
        + 8   // adl_trigger_count
        + 8   // balance_1h_ago_e6
        + 8   // last_snapshot_ts
        + 8   // withdrawal_delay_secs
        + 1   // is_adl_in_progress
        + 32  // authorized_caller
        + 8   // last_update_ts
        + 64; // reserved
    
    /// 创建新的 InsuranceFundConfig
    pub fn new(
        fund: Pubkey,
        bump: u8,
        adl_trigger_threshold_e6: i64,
        withdrawal_delay_secs: i64,
        authorized_caller: Pubkey,
        created_at: i64,
    ) -> Self {
        Self {
            discriminator: INSURANCE_FUND_CONFIG_DISCRIMINATOR,
            fund,
            bump,
            total_liquidation_income_e6: 0,
            total_adl_profit_e6: 0,
            total_shortfall_payout_e6: 0,
            adl_trigger_threshold_e6,
            adl_trigger_count: 0,
            balance_1h_ago_e6: 0,
            last_snapshot_ts: created_at,
            withdrawal_delay_secs,
            is_adl_in_progress: false,
            authorized_caller,
            last_update_ts: created_at,
            reserved: [0u8; 64],
        }
    }
    
    /// PDA seeds for InsuranceFundConfig
    pub fn seeds() -> Vec<Vec<u8>> {
        vec![INSURANCE_FUND_CONFIG_SEED.to_vec()]
    }
    
    /// 检查是否需要触发 ADL
    /// 
    /// 三重触发条件:
    /// 1. 穿仓触发: 保险基金余额 < 需要覆盖的穿仓金额
    /// 2. 余额不足触发: 保险基金余额 < 最低阈值
    /// 3. 1小时下降30%触发: 当前余额 < 1小时前余额 * 70%
    pub fn should_trigger_adl(&self, current_balance_e6: i64, shortfall_e6: i64) -> ADLTriggerReason {
        // 条件1: 穿仓触发
        if shortfall_e6 > 0 && current_balance_e6 < shortfall_e6 {
            return ADLTriggerReason::Bankruptcy;
        }
        
        // 条件2: 余额不足触发
        if current_balance_e6 < self.adl_trigger_threshold_e6 {
            return ADLTriggerReason::InsufficientBalance;
        }
        
        // 条件3: 1小时下降30%触发
        // 只有在有历史数据时才检查
        if self.balance_1h_ago_e6 > 0 {
            let threshold_70_percent = self.balance_1h_ago_e6 * 70 / 100;
            if current_balance_e6 < threshold_70_percent {
                return ADLTriggerReason::RapidDecline;
            }
        }
        
        ADLTriggerReason::None
    }
    
    /// 覆盖穿仓损失
    /// 
    /// 返回: (实际覆盖金额, 剩余穿仓金额)
    /// 如果剩余穿仓金额 > 0，需要触发 ADL
    pub fn cover_shortfall(&mut self, shortfall_e6: i64, current_balance_e6: i64) -> (i64, i64) {
        if shortfall_e6 <= current_balance_e6 {
            // 保险基金可以完全覆盖
            self.total_shortfall_payout_e6 = self.total_shortfall_payout_e6.saturating_add(shortfall_e6);
            (shortfall_e6, 0)
        } else {
            // 保险基金不足，返回剩余穿仓金额
            let covered = current_balance_e6;
            let remaining = shortfall_e6.saturating_sub(covered);
            self.total_shortfall_payout_e6 = self.total_shortfall_payout_e6.saturating_add(covered);
            (covered, remaining)
        }
    }
    
    /// 添加清算收入
    pub fn add_liquidation_income(&mut self, amount_e6: i64) {
        self.total_liquidation_income_e6 = self.total_liquidation_income_e6.saturating_add(amount_e6);
    }
    
    /// 添加 ADL 盈余
    pub fn add_adl_profit(&mut self, amount_e6: i64) {
        self.total_adl_profit_e6 = self.total_adl_profit_e6.saturating_add(amount_e6);
    }
    
    /// 添加交易手续费收入 (V1 简化方案: 记入 liquidation_income)
    /// 
    /// V1: 手续费直接计入 total_liquidation_income_e6 统一管理
    /// V2: 可扩展为单独的 total_trading_fee_e6 字段 (使用 reserved bytes)
    pub fn add_trading_fee(&mut self, fee_e6: i64) {
        // V1: 简化方案 - 手续费与清算收入一起记账
        self.total_liquidation_income_e6 = self.total_liquidation_income_e6.saturating_add(fee_e6);
    }
    
    /// 更新1小时快照
    pub fn update_hourly_snapshot(&mut self, current_balance_e6: i64, current_ts: i64) {
        self.balance_1h_ago_e6 = current_balance_e6;
        self.last_snapshot_ts = current_ts;
    }
    
    /// 设置 ADL 进行中状态
    pub fn set_adl_in_progress(&mut self, in_progress: bool) {
        self.is_adl_in_progress = in_progress;
        if in_progress {
            self.adl_trigger_count = self.adl_trigger_count.saturating_add(1);
        }
    }
    
    /// 检查是否允许 LP 赎回
    pub fn can_withdraw(&self) -> bool {
        !self.is_adl_in_progress
    }
    
    /// 验证调用方是否授权
    pub fn is_authorized_caller(&self, caller: &Pubkey) -> bool {
        caller == &self.authorized_caller
    }
    
    /// 获取总收入
    pub fn total_income_e6(&self) -> i64 {
        self.total_liquidation_income_e6.saturating_add(self.total_adl_profit_e6)
    }
    
    /// 获取净收入 (收入 - 支出)
    pub fn net_income_e6(&self) -> i64 {
        self.total_income_e6().saturating_sub(self.total_shortfall_payout_e6)
    }
}

// =============================================================================
// Square Payment Record
// =============================================================================

/// Square 支付类型
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquarePaymentType {
    /// 知识付费买断
    KnowledgePurchase = 0,
    /// 月度订阅
    Subscription = 1,
    /// 直播打赏
    LiveDonation = 2,
}

impl Default for SquarePaymentType {
    fn default() -> Self {
        SquarePaymentType::KnowledgePurchase
    }
}

/// Square 平台支付记录
/// 
/// 记录 Square 平台上的所有支付交易，包括：
/// - 知识付费买断
/// - 月度订阅
/// - 直播打赏
/// 
/// 资金分成: 一部分进入创作者 Vault，一部分进入平台 Square Fund
/// 
/// PDA Seeds: ["square_payment", payer, content_id, timestamp]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SquarePaymentRecord {
    /// 账户类型标识符
    pub discriminator: u64,
    
    /// 支付者地址 (用户)
    pub payer: Pubkey,
    
    /// 创作者地址 (收款人)
    pub creator: Pubkey,
    
    /// 内容 ID (唯一标识内容)
    pub content_id: u64,
    
    /// 支付类型
    pub payment_type: SquarePaymentType,
    
    /// 总支付金额 (e6)
    pub total_amount_e6: i64,
    
    /// 创作者分成金额 (e6) - 进入创作者 Vault
    pub creator_amount_e6: i64,
    
    /// 平台分成金额 (e6) - 进入 Square Fund
    pub platform_amount_e6: i64,
    
    /// 创作者分成比例 (基点, 10000 = 100%)
    pub creator_share_bps: u16,
    
    /// 支付时间戳
    pub payment_ts: i64,
    
    /// 订阅周期数 (仅用于订阅类型)
    pub subscription_period: u8,
    
    /// 交易备注 (最多32字节)
    pub memo: [u8; 32],
    
    /// PDA bump
    pub bump: u8,
    
    /// 保留字段
    pub reserved: [u8; 16],
}

impl SquarePaymentRecord {
    /// Account size in bytes
    pub const SIZE: usize = 8    // discriminator
        + 32  // payer
        + 32  // creator
        + 8   // content_id
        + 1   // payment_type
        + 8   // total_amount_e6
        + 8   // creator_amount_e6
        + 8   // platform_amount_e6
        + 2   // creator_share_bps
        + 8   // payment_ts
        + 1   // subscription_period
        + 32  // memo
        + 1   // bump
        + 16; // reserved
    
    /// 创建新的支付记录
    pub fn new(
        payer: Pubkey,
        creator: Pubkey,
        content_id: u64,
        payment_type: SquarePaymentType,
        total_amount_e6: i64,
        creator_share_bps: u16,
        payment_ts: i64,
        subscription_period: u8,
        memo: &[u8],
        bump: u8,
    ) -> Self {
        // 计算分成金额
        let creator_amount_e6 = (total_amount_e6 as i128 * creator_share_bps as i128 / 10000) as i64;
        let platform_amount_e6 = total_amount_e6.saturating_sub(creator_amount_e6);
        
        let mut memo_array = [0u8; 32];
        let copy_len = memo.len().min(32);
        memo_array[..copy_len].copy_from_slice(&memo[..copy_len]);
        
        Self {
            discriminator: SQUARE_PAYMENT_RECORD_DISCRIMINATOR,
            payer,
            creator,
            content_id,
            payment_type,
            total_amount_e6,
            creator_amount_e6,
            platform_amount_e6,
            creator_share_bps,
            payment_ts,
            subscription_period,
            memo: memo_array,
            bump,
            reserved: [0u8; 16],
        }
    }
    
    /// PDA seeds for SquarePaymentRecord
    pub fn seeds(payer: &Pubkey, content_id: u64, timestamp: i64) -> Vec<Vec<u8>> {
        vec![
            SQUARE_PAYMENT_RECORD_SEED.to_vec(),
            payer.to_bytes().to_vec(),
            content_id.to_le_bytes().to_vec(),
            timestamp.to_le_bytes().to_vec(),
        ]
    }
    
    /// 获取创作者分成金额
    pub fn get_creator_amount(&self) -> i64 {
        self.creator_amount_e6
    }
    
    /// 获取平台分成金额
    pub fn get_platform_amount(&self) -> i64 {
        self.platform_amount_e6
    }
    
    /// 检查是否为订阅类型
    pub fn is_subscription(&self) -> bool {
        self.payment_type == SquarePaymentType::Subscription
    }
    
    /// 获取 memo 字符串
    pub fn memo_str(&self) -> &str {
        let end = self.memo.iter().position(|&b| b == 0).unwrap_or(32);
        std::str::from_utf8(&self.memo[..end]).unwrap_or("")
    }
}

// =============================================================================
// Referral System
// =============================================================================

/// 最大邀请码长度
pub const MAX_REFERRAL_CODE_LEN: usize = 12;

/// VIP 等级数量
pub const VIP_LEVELS: usize = 6;

/// 默认邀请人分成 (2000 = 20%)
pub const DEFAULT_REFERRER_SHARE_BPS: u16 = 2000;

/// 默认被邀请人折扣 (1000 = 10%)
pub const DEFAULT_REFEREE_DISCOUNT_BPS: u16 = 1000;

/// 全局返佣配置
/// 
/// PDA Seeds: ["referral_config"]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ReferralConfig {
    /// 账户类型标识
    pub discriminator: u64,
    
    /// 管理员
    pub authority: Pubkey,
    
    /// Vault Program ID (用于 CPI 转账)
    pub vault_program: Pubkey,
    
    // === 基础分成比例 (basis points, 10000 = 100%) ===
    
    /// 邀请人获得手续费的比例 (默认 2000 = 20%)
    pub referrer_share_bps: u16,
    
    /// 被邀请人手续费折扣 (默认 1000 = 10%)
    pub referee_discount_bps: u16,
    
    // === VIP 等级加成 ===
    
    /// 邀请人 VIP 等级加成 [VIP0, VIP1, ..., VIP5] bps
    pub referrer_vip_bonus_bps: [u16; VIP_LEVELS],
    
    /// 被邀请人 VIP 等级折扣加成 [VIP0, VIP1, ..., VIP5] bps
    pub referee_vip_bonus_bps: [u16; VIP_LEVELS],
    
    // === 限制 ===
    
    /// 最低结算金额 (e6) - 低于此金额累计
    pub min_settlement_amount_e6: i64,
    
    /// 返佣有效期 (秒) - 0 = 永久
    pub reward_validity_secs: i64,
    
    // === 统计 ===
    
    /// 总发放返佣金额 (e6)
    pub total_rewards_paid_e6: i64,
    
    /// 总发放折扣金额 (e6)
    pub total_discounts_given_e6: i64,
    
    /// 总注册邀请链接数
    pub total_referral_links: u64,
    
    /// 总邀请用户数
    pub total_referred_users: u64,
    
    /// 总产生交易量 (e6)
    pub total_referred_volume_e6: i64,
    
    // === 状态 ===
    
    /// 是否暂停
    pub is_paused: bool,
    
    /// PDA bump
    pub bump: u8,
    
    /// 最后更新时间
    pub last_update_ts: i64,
    
    /// 预留字段
    pub reserved: [u8; 64],
}

impl ReferralConfig {
    /// 账户大小
    pub const SIZE: usize = 8   // discriminator
        + 32  // authority
        + 32  // vault_program
        + 2   // referrer_share_bps
        + 2   // referee_discount_bps
        + 12  // referrer_vip_bonus_bps (6 * 2)
        + 12  // referee_vip_bonus_bps (6 * 2)
        + 8   // min_settlement_amount_e6
        + 8   // reward_validity_secs
        + 8   // total_rewards_paid_e6
        + 8   // total_discounts_given_e6
        + 8   // total_referral_links
        + 8   // total_referred_users
        + 8   // total_referred_volume_e6
        + 1   // is_paused
        + 1   // bump
        + 8   // last_update_ts
        + 64; // reserved
    
    /// 创建新的 ReferralConfig
    pub fn new(
        authority: Pubkey,
        vault_program: Pubkey,
        referrer_share_bps: u16,
        referee_discount_bps: u16,
        bump: u8,
        created_at: i64,
    ) -> Self {
        Self {
            discriminator: REFERRAL_CONFIG_DISCRIMINATOR,
            authority,
            vault_program,
            referrer_share_bps,
            referee_discount_bps,
            // 默认 VIP 加成: [0%, 2%, 5%, 10%, 15%, 20%]
            referrer_vip_bonus_bps: [0, 200, 500, 1000, 1500, 2000],
            referee_vip_bonus_bps: [0, 200, 500, 1000, 1500, 2000],
            min_settlement_amount_e6: 10_000_000, // $10 最低结算
            reward_validity_secs: 0, // 永久有效
            total_rewards_paid_e6: 0,
            total_discounts_given_e6: 0,
            total_referral_links: 0,
            total_referred_users: 0,
            total_referred_volume_e6: 0,
            is_paused: false,
            bump,
            last_update_ts: created_at,
            reserved: [0u8; 64],
        }
    }
    
    /// PDA seeds
    pub fn seeds() -> Vec<Vec<u8>> {
        vec![REFERRAL_CONFIG_SEED.to_vec()]
    }
    
    /// 获取邀请人总分成比例 (基础 + VIP 加成)
    pub fn get_referrer_share(&self, vip_level: u8) -> u16 {
        let level = (vip_level as usize).min(VIP_LEVELS - 1);
        self.referrer_share_bps.saturating_add(self.referrer_vip_bonus_bps[level])
    }
    
    /// 获取被邀请人总折扣比例 (基础 + VIP 加成)
    pub fn get_referee_discount(&self, vip_level: u8) -> u16 {
        let level = (vip_level as usize).min(VIP_LEVELS - 1);
        self.referee_discount_bps.saturating_add(self.referee_vip_bonus_bps[level])
    }
    
    /// 计算返佣金额
    /// 
    /// 返回: (referrer_reward, referee_discount, platform_income)
    pub fn calculate_rewards(
        &self,
        trade_fee_e6: i64,
        referrer_vip: u8,
        referee_vip: u8,
    ) -> (i64, i64, i64) {
        // 取较高的 VIP 等级
        let effective_vip = referrer_vip.max(referee_vip);
        
        // 计算被邀请人折扣
        let discount_bps = self.get_referee_discount(effective_vip);
        let referee_discount = (trade_fee_e6 as i128 * discount_bps as i128 / 10000) as i64;
        
        // 实际收取的手续费
        let actual_fee = trade_fee_e6.saturating_sub(referee_discount);
        
        // 计算邀请人返佣 (基于实际收取的手续费)
        let referrer_share_bps = self.get_referrer_share(effective_vip);
        let referrer_reward = (actual_fee as i128 * referrer_share_bps as i128 / 10000) as i64;
        
        // 平台收入
        let platform_income = actual_fee.saturating_sub(referrer_reward);
        
        (referrer_reward, referee_discount, platform_income)
    }
    
    /// 更新统计
    pub fn record_reward(
        &mut self,
        referrer_reward_e6: i64,
        referee_discount_e6: i64,
        volume_e6: i64,
        current_ts: i64,
    ) {
        self.total_rewards_paid_e6 = self.total_rewards_paid_e6.saturating_add(referrer_reward_e6);
        self.total_discounts_given_e6 = self.total_discounts_given_e6.saturating_add(referee_discount_e6);
        self.total_referred_volume_e6 = self.total_referred_volume_e6.saturating_add(volume_e6);
        self.last_update_ts = current_ts;
    }
}

/// 邀请链接
/// 
/// PDA Seeds: ["referral_link", referrer]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ReferralLink {
    /// 账户类型标识
    pub discriminator: u64,
    
    /// 邀请人
    pub referrer: Pubkey,
    
    /// 邀请码 (唯一, 6-12 字符)
    pub code: [u8; MAX_REFERRAL_CODE_LEN],
    
    /// 创建时间
    pub created_at: i64,
    
    /// 是否激活
    pub is_active: bool,
    
    // === 自定义配置 (可选) ===
    
    /// 自定义邀请人分成 (0 = 使用全局配置)
    pub custom_referrer_share_bps: u16,
    
    /// 自定义被邀请人折扣 (0 = 使用全局配置)
    pub custom_referee_discount_bps: u16,
    
    // === 统计 ===
    
    /// 邀请人数
    pub referred_count: u32,
    
    /// 累计交易量 (被邀请人产生)
    pub total_volume_e6: i64,
    
    /// 累计获得返佣
    pub total_rewards_earned_e6: i64,
    
    /// 累计发放折扣
    pub total_discounts_given_e6: i64,
    
    /// PDA bump
    pub bump: u8,
    
    /// 预留字段
    pub reserved: [u8; 32],
}

impl ReferralLink {
    /// 账户大小
    pub const SIZE: usize = 8   // discriminator
        + 32  // referrer
        + MAX_REFERRAL_CODE_LEN  // code
        + 8   // created_at
        + 1   // is_active
        + 2   // custom_referrer_share_bps
        + 2   // custom_referee_discount_bps
        + 4   // referred_count
        + 8   // total_volume_e6
        + 8   // total_rewards_earned_e6
        + 8   // total_discounts_given_e6
        + 1   // bump
        + 32; // reserved
    
    /// 创建新的邀请链接
    pub fn new(
        referrer: Pubkey,
        code: &[u8],
        bump: u8,
        created_at: i64,
    ) -> Self {
        let mut code_bytes = [0u8; MAX_REFERRAL_CODE_LEN];
        let len = code.len().min(MAX_REFERRAL_CODE_LEN);
        code_bytes[..len].copy_from_slice(&code[..len]);
        
        Self {
            discriminator: REFERRAL_LINK_DISCRIMINATOR,
            referrer,
            code: code_bytes,
            created_at,
            is_active: true,
            custom_referrer_share_bps: 0,
            custom_referee_discount_bps: 0,
            referred_count: 0,
            total_volume_e6: 0,
            total_rewards_earned_e6: 0,
            total_discounts_given_e6: 0,
            bump,
            reserved: [0u8; 32],
        }
    }
    
    /// PDA seeds
    pub fn seeds(referrer: &Pubkey) -> Vec<Vec<u8>> {
        vec![
            REFERRAL_LINK_SEED.to_vec(),
            referrer.to_bytes().to_vec(),
        ]
    }
    
    /// 获取邀请码字符串
    pub fn code_str(&self) -> String {
        let end = self.code.iter().position(|&b| b == 0).unwrap_or(self.code.len());
        String::from_utf8_lossy(&self.code[..end]).to_string()
    }
    
    /// 记录新邀请
    pub fn record_referral(&mut self) {
        self.referred_count = self.referred_count.saturating_add(1);
    }
    
    /// 记录返佣
    pub fn record_reward(&mut self, reward_e6: i64, discount_e6: i64, volume_e6: i64) {
        self.total_rewards_earned_e6 = self.total_rewards_earned_e6.saturating_add(reward_e6);
        self.total_discounts_given_e6 = self.total_discounts_given_e6.saturating_add(discount_e6);
        self.total_volume_e6 = self.total_volume_e6.saturating_add(volume_e6);
    }
}

/// 邀请关系绑定
/// 
/// PDA Seeds: ["referral_binding", referee]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ReferralBinding {
    /// 账户类型标识
    pub discriminator: u64,
    
    /// 被邀请人
    pub referee: Pubkey,
    
    /// 邀请人
    pub referrer: Pubkey,
    
    /// 邀请链接
    pub referral_link: Pubkey,
    
    /// 绑定时间
    pub bound_at: i64,
    
    // === 统计 ===
    
    /// 被邀请人累计交易量 (e6)
    pub referee_volume_e6: i64,
    
    /// 邀请人从此用户获得的返佣 (e6)
    pub referrer_rewards_e6: i64,
    
    /// 被邀请人获得的折扣 (e6)
    pub referee_discounts_e6: i64,
    
    /// 交易次数
    pub trade_count: u64,
    
    /// 最后交易时间
    pub last_trade_ts: i64,
    
    /// PDA bump
    pub bump: u8,
    
    /// 预留字段
    pub reserved: [u8; 32],
}

impl ReferralBinding {
    /// 账户大小
    pub const SIZE: usize = 8   // discriminator
        + 32  // referee
        + 32  // referrer
        + 32  // referral_link
        + 8   // bound_at
        + 8   // referee_volume_e6
        + 8   // referrer_rewards_e6
        + 8   // referee_discounts_e6
        + 8   // trade_count
        + 8   // last_trade_ts
        + 1   // bump
        + 32; // reserved
    
    /// 创建新的邀请关系
    pub fn new(
        referee: Pubkey,
        referrer: Pubkey,
        referral_link: Pubkey,
        bump: u8,
        bound_at: i64,
    ) -> Self {
        Self {
            discriminator: REFERRAL_BINDING_DISCRIMINATOR,
            referee,
            referrer,
            referral_link,
            bound_at,
            referee_volume_e6: 0,
            referrer_rewards_e6: 0,
            referee_discounts_e6: 0,
            trade_count: 0,
            last_trade_ts: 0,
            bump,
            reserved: [0u8; 32],
        }
    }
    
    /// PDA seeds
    pub fn seeds(referee: &Pubkey) -> Vec<Vec<u8>> {
        vec![
            REFERRAL_BINDING_SEED.to_vec(),
            referee.to_bytes().to_vec(),
        ]
    }
    
    /// 记录交易
    pub fn record_trade(
        &mut self,
        volume_e6: i64,
        referrer_reward_e6: i64,
        referee_discount_e6: i64,
        current_ts: i64,
    ) {
        self.referee_volume_e6 = self.referee_volume_e6.saturating_add(volume_e6);
        self.referrer_rewards_e6 = self.referrer_rewards_e6.saturating_add(referrer_reward_e6);
        self.referee_discounts_e6 = self.referee_discounts_e6.saturating_add(referee_discount_e6);
        self.trade_count = self.trade_count.saturating_add(1);
        self.last_trade_ts = current_ts;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_fund_config_size() {
        assert!(FundConfig::SIZE > 0);
        println!("FundConfig SIZE: {}", FundConfig::SIZE);
    }

    #[test]
    fn test_fund_size() {
        assert!(Fund::SIZE > 0);
        println!("Fund SIZE: {}", Fund::SIZE);
    }

    #[test]
    fn test_lp_position_size() {
        assert!(LPPosition::SIZE > 0);
        println!("LPPosition SIZE: {}", LPPosition::SIZE);
    }

    #[test]
    fn test_fund_creation() {
        let manager = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let fee_config = FeeConfig::new(200, 2000);
        
        let fund = Fund::new(
            manager,
            "Test Fund",
            254,
            vault,
            mint,
            fee_config,
            1,
            1000000,
        );
        
        assert_eq!(fund.manager, manager);
        assert_eq!(fund.name_str(), "Test Fund");
        assert!(fund.is_open);
        assert!(!fund.is_paused);
        assert_eq!(fund.stats.current_nav_e6, INITIAL_NAV_E6);
    }

    #[test]
    fn test_fund_deposit_withdrawal() {
        let manager = Pubkey::new_unique();
        let vault = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let fee_config = FeeConfig::new(200, 2000);
        
        let mut fund = Fund::new(
            manager,
            "Test Fund",
            254,
            vault,
            mint,
            fee_config,
            1,
            1000000,
        );
        
        // Record deposit
        fund.record_deposit(100_000_000, 100_000_000).unwrap();
        assert_eq!(fund.stats.total_deposits_e6, 100_000_000);
        assert_eq!(fund.stats.total_shares, 100_000_000);
        
        // Record withdrawal
        fund.record_withdrawal(50_000_000, 50_000_000).unwrap();
        assert_eq!(fund.stats.total_withdrawals_e6, 50_000_000);
        assert_eq!(fund.stats.total_shares, 50_000_000);
    }

    #[test]
    fn test_lp_position() {
        let fund = Pubkey::new_unique();
        let investor = Pubkey::new_unique();
        
        let mut position = LPPosition::new(
            fund,
            investor,
            100_000_000, // 100 shares
            1_000_000,   // NAV = 1.0
            100_000_000, // 100 USDC
            1000000,
            254,
        );
        
        // Check current value at NAV = 1.0
        assert_eq!(position.current_value(1_000_000), 100_000_000);
        
        // Check current value at NAV = 1.5
        assert_eq!(position.current_value(1_500_000), 150_000_000);
        
        // Check unrealized PnL at NAV = 1.5
        assert_eq!(position.unrealized_pnl(1_500_000), 50_000_000);
        
        // Add more shares
        position.add_shares(50_000_000, 50_000_000, 1_000_000, 2000000).unwrap();
        assert_eq!(position.shares, 150_000_000);
        assert_eq!(position.total_deposited_e6, 150_000_000);
        
        // Remove shares
        position.remove_shares(25_000_000, 25_000_000, 3000000).unwrap();
        assert_eq!(position.shares, 125_000_000);
        assert_eq!(position.total_withdrawn_e6, 25_000_000);
    }

    #[test]
    fn test_fund_stats() {
        let mut stats = FundStats::new(1000000);
        
        assert_eq!(stats.current_nav_e6, INITIAL_NAV_E6);
        assert_eq!(stats.high_water_mark_e6, INITIAL_NAV_E6);
        assert_eq!(stats.total_shares, 0);
        
        // Simulate deposits
        stats.total_deposits_e6 = 100_000_000;
        stats.total_shares = 100_000_000;
        stats.update_nav().unwrap();
        
        assert_eq!(stats.current_nav_e6, 1_000_000); // NAV = 1.0
        
        // Simulate profit
        stats.total_realized_pnl_e6 = 20_000_000;
        stats.update_nav().unwrap();
        stats.update_hwm();
        
        // NAV should increase: (100 - 0 + 20 - 0 - 0) / 100 = 1.2
        assert_eq!(stats.current_nav_e6, 1_200_000);
        assert_eq!(stats.high_water_mark_e6, 1_200_000);
    }

    // === Insurance Fund Config Tests ===

    #[test]
    fn test_insurance_fund_config_size() {
        assert!(InsuranceFundConfig::SIZE > 0);
        println!("InsuranceFundConfig SIZE: {}", InsuranceFundConfig::SIZE);
    }

    #[test]
    fn test_insurance_fund_config_creation() {
        let fund = Pubkey::new_unique();
        let caller = Pubkey::new_unique();
        
        let config = InsuranceFundConfig::new(
            fund,
            254,
            100_000_000,      // 100 USDC threshold
            3600,             // 1 hour delay
            caller,
            1000000,
        );
        
        assert_eq!(config.fund, fund);
        assert_eq!(config.adl_trigger_threshold_e6, 100_000_000);
        assert_eq!(config.withdrawal_delay_secs, 3600);
        assert_eq!(config.total_liquidation_income_e6, 0);
        assert!(!config.is_adl_in_progress);
    }

    #[test]
    fn test_insurance_fund_adl_trigger_conditions() {
        let fund = Pubkey::new_unique();
        let caller = Pubkey::new_unique();
        
        let mut config = InsuranceFundConfig::new(
            fund,
            254,
            100_000_000,      // 100 USDC threshold
            3600,
            caller,
            1000000,
        );
        
        // 设置1小时前余额
        config.balance_1h_ago_e6 = 1000_000_000; // 1000 USDC
        
        // 测试条件1: 穿仓触发
        assert_eq!(
            config.should_trigger_adl(50_000_000, 100_000_000), // 余额50, 穿仓100
            ADLTriggerReason::Bankruptcy
        );
        
        // 测试条件2: 余额不足触发
        assert_eq!(
            config.should_trigger_adl(50_000_000, 0), // 余额50 < 阈值100
            ADLTriggerReason::InsufficientBalance
        );
        
        // 测试条件3: 1小时下降30%触发
        assert_eq!(
            config.should_trigger_adl(600_000_000, 0), // 余额600 < 1000*0.7=700
            ADLTriggerReason::RapidDecline
        );
        
        // 测试正常情况: 不触发
        assert_eq!(
            config.should_trigger_adl(800_000_000, 0), // 余额800 > 阈值100, > 700
            ADLTriggerReason::None
        );
    }

    #[test]
    fn test_insurance_fund_cover_shortfall() {
        let fund = Pubkey::new_unique();
        let caller = Pubkey::new_unique();
        
        let mut config = InsuranceFundConfig::new(
            fund,
            254,
            100_000_000,
            3600,
            caller,
            1000000,
        );
        
        // 情况1: 完全覆盖
        let (covered, remaining) = config.cover_shortfall(500_000_000, 1000_000_000);
        assert_eq!(covered, 500_000_000);
        assert_eq!(remaining, 0);
        assert_eq!(config.total_shortfall_payout_e6, 500_000_000);
        
        // 情况2: 部分覆盖
        let (covered, remaining) = config.cover_shortfall(600_000_000, 400_000_000);
        assert_eq!(covered, 400_000_000);
        assert_eq!(remaining, 200_000_000);
        assert_eq!(config.total_shortfall_payout_e6, 900_000_000);
    }

    #[test]
    fn test_insurance_fund_income_tracking() {
        let fund = Pubkey::new_unique();
        let caller = Pubkey::new_unique();
        
        let mut config = InsuranceFundConfig::new(
            fund,
            254,
            100_000_000,
            3600,
            caller,
            1000000,
        );
        
        // 添加清算收入
        config.add_liquidation_income(100_000_000);
        assert_eq!(config.total_liquidation_income_e6, 100_000_000);
        
        // 添加ADL盈余
        config.add_adl_profit(50_000_000);
        assert_eq!(config.total_adl_profit_e6, 50_000_000);
        
        // 检查总收入
        assert_eq!(config.total_income_e6(), 150_000_000);
        
        // 模拟支出
        config.cover_shortfall(30_000_000, 1000_000_000);
        
        // 检查净收入
        assert_eq!(config.net_income_e6(), 120_000_000); // 150 - 30
    }

    // === Square Payment Record Tests ===

    #[test]
    fn test_square_payment_record_size() {
        assert!(SquarePaymentRecord::SIZE > 0);
        println!("SquarePaymentRecord SIZE: {}", SquarePaymentRecord::SIZE);
    }

    #[test]
    fn test_square_payment_record_creation() {
        let payer = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        let content_id = 12345u64;
        let payment_type = SquarePaymentType::KnowledgePurchase;
        let total_amount = 100_000_000i64; // 100 USDC
        let creator_share_bps = 9000u16; // 90%
        let timestamp = 1700000000i64;
        
        let record = SquarePaymentRecord::new(
            payer,
            creator,
            content_id,
            payment_type,
            total_amount,
            creator_share_bps,
            timestamp,
            0, // no subscription period
            b"Test payment",
            254,
        );
        
        assert_eq!(record.payer, payer);
        assert_eq!(record.creator, creator);
        assert_eq!(record.content_id, content_id);
        assert_eq!(record.payment_type, SquarePaymentType::KnowledgePurchase);
        assert_eq!(record.total_amount_e6, 100_000_000);
        assert_eq!(record.creator_amount_e6, 90_000_000); // 90%
        assert_eq!(record.platform_amount_e6, 10_000_000); // 10%
        assert_eq!(record.creator_share_bps, 9000);
        assert_eq!(record.payment_ts, timestamp);
        assert!(!record.is_subscription());
    }

    #[test]
    fn test_square_payment_subscription() {
        let payer = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        
        let record = SquarePaymentRecord::new(
            payer,
            creator,
            99999,
            SquarePaymentType::Subscription,
            50_000_000, // 50 USDC
            8500, // 85%
            1700000000,
            12, // 12 months
            b"Monthly sub",
            254,
        );
        
        assert!(record.is_subscription());
        assert_eq!(record.subscription_period, 12);
        assert_eq!(record.creator_amount_e6, 42_500_000); // 85%
        assert_eq!(record.platform_amount_e6, 7_500_000); // 15%
    }

    #[test]
    fn test_square_payment_live_donation() {
        let payer = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        
        let record = SquarePaymentRecord::new(
            payer,
            creator,
            1,
            SquarePaymentType::LiveDonation,
            10_000_000, // 10 USDC
            7000, // 70%
            1700000000,
            0,
            b"Great stream!",
            254,
        );
        
        assert_eq!(record.payment_type, SquarePaymentType::LiveDonation);
        assert_eq!(record.creator_amount_e6, 7_000_000); // 70%
        assert_eq!(record.platform_amount_e6, 3_000_000); // 30%
        assert_eq!(record.memo_str(), "Great stream!");
    }

    #[test]
    fn test_square_payment_memo_truncation() {
        let payer = Pubkey::new_unique();
        let creator = Pubkey::new_unique();
        
        // Test with a long memo that should be truncated
        let long_memo = b"This is a very long memo that exceeds 32 bytes and should be truncated";
        let record = SquarePaymentRecord::new(
            payer,
            creator,
            1,
            SquarePaymentType::KnowledgePurchase,
            10_000_000,
            9000,
            1700000000,
            0,
            long_memo,
            254,
        );
        
        // Memo should be truncated to 32 bytes
        assert_eq!(record.memo.len(), 32);
        // First 32 bytes should match
        assert_eq!(&record.memo[..], &long_memo[..32]);
    }

    #[test]
    fn test_square_payment_seeds() {
        let payer = Pubkey::new_unique();
        let content_id = 12345u64;
        let timestamp = 1700000000i64;
        
        let seeds = SquarePaymentRecord::seeds(&payer, content_id, timestamp);
        
        assert_eq!(seeds.len(), 4);
        assert_eq!(seeds[0], SQUARE_PAYMENT_RECORD_SEED.to_vec());
        assert_eq!(seeds[1], payer.to_bytes().to_vec());
        assert_eq!(seeds[2], content_id.to_le_bytes().to_vec());
        assert_eq!(seeds[3], timestamp.to_le_bytes().to_vec());
    }

    // === Referral Config Tests ===

    #[test]
    fn test_referral_config_size() {
        assert!(ReferralConfig::SIZE > 0);
        println!("ReferralConfig SIZE: {}", ReferralConfig::SIZE);
    }

    #[test]
    fn test_referral_config_creation() {
        let authority = Pubkey::new_unique();
        let vault_program = Pubkey::new_unique();
        
        let config = ReferralConfig::new(
            authority,
            vault_program,
            DEFAULT_REFERRER_SHARE_BPS,  // 20%
            DEFAULT_REFEREE_DISCOUNT_BPS, // 10%
            254,
            1700000000,
        );
        
        assert_eq!(config.authority, authority);
        assert_eq!(config.referrer_share_bps, 2000);
        assert_eq!(config.referee_discount_bps, 1000);
        assert!(!config.is_paused);
        assert_eq!(config.total_referral_links, 0);
    }

    #[test]
    fn test_referral_config_vip_bonus() {
        let authority = Pubkey::new_unique();
        let vault_program = Pubkey::new_unique();
        
        let config = ReferralConfig::new(
            authority,
            vault_program,
            2000, // 20%
            1000, // 10%
            254,
            1700000000,
        );
        
        // VIP 0: 20% base + 0% bonus = 20%
        assert_eq!(config.get_referrer_share(0), 2000);
        assert_eq!(config.get_referee_discount(0), 1000);
        
        // VIP 3: 20% base + 10% bonus = 30%
        assert_eq!(config.get_referrer_share(3), 3000);
        assert_eq!(config.get_referee_discount(3), 2000);
        
        // VIP 5: 20% base + 20% bonus = 40%
        assert_eq!(config.get_referrer_share(5), 4000);
        assert_eq!(config.get_referee_discount(5), 3000);
    }

    #[test]
    fn test_referral_reward_calculation() {
        let authority = Pubkey::new_unique();
        let vault_program = Pubkey::new_unique();
        
        let config = ReferralConfig::new(
            authority,
            vault_program,
            2000, // 20%
            1000, // 10%
            254,
            1700000000,
        );
        
        // 测试: $100 手续费, VIP 0
        let (referrer_reward, referee_discount, platform_income) = 
            config.calculate_rewards(100_000_000, 0, 0);
        
        // 被邀请人折扣: $100 * 10% = $10
        assert_eq!(referee_discount, 10_000_000);
        // 实际收费: $100 - $10 = $90
        // 邀请人返佣: $90 * 20% = $18
        assert_eq!(referrer_reward, 18_000_000);
        // 平台收入: $90 - $18 = $72
        assert_eq!(platform_income, 72_000_000);
        
        // 测试: $100 手续费, VIP 3 (取较高)
        let (referrer_reward, referee_discount, platform_income) = 
            config.calculate_rewards(100_000_000, 3, 1);
        
        // VIP 3 折扣: 10% + 10% = 20%
        // 被邀请人折扣: $100 * 20% = $20
        assert_eq!(referee_discount, 20_000_000);
        // 实际收费: $100 - $20 = $80
        // VIP 3 分成: 20% + 10% = 30%
        // 邀请人返佣: $80 * 30% = $24
        assert_eq!(referrer_reward, 24_000_000);
        // 平台收入: $80 - $24 = $56
        assert_eq!(platform_income, 56_000_000);
    }

    // === Referral Link Tests ===

    #[test]
    fn test_referral_link_size() {
        assert!(ReferralLink::SIZE > 0);
        println!("ReferralLink SIZE: {}", ReferralLink::SIZE);
    }

    #[test]
    fn test_referral_link_creation() {
        let referrer = Pubkey::new_unique();
        let code = b"ALICE2024";
        
        let link = ReferralLink::new(referrer, code, 254, 1700000000);
        
        assert_eq!(link.referrer, referrer);
        assert_eq!(link.code_str(), "ALICE2024");
        assert!(link.is_active);
        assert_eq!(link.referred_count, 0);
        assert_eq!(link.total_rewards_earned_e6, 0);
    }

    #[test]
    fn test_referral_link_statistics() {
        let referrer = Pubkey::new_unique();
        let mut link = ReferralLink::new(referrer, b"TEST123", 254, 1700000000);
        
        // 记录新邀请
        link.record_referral();
        assert_eq!(link.referred_count, 1);
        
        // 记录返佣
        link.record_reward(18_000_000, 10_000_000, 1000_000_000);
        assert_eq!(link.total_rewards_earned_e6, 18_000_000);
        assert_eq!(link.total_discounts_given_e6, 10_000_000);
        assert_eq!(link.total_volume_e6, 1000_000_000);
    }

    // === Referral Binding Tests ===

    #[test]
    fn test_referral_binding_size() {
        assert!(ReferralBinding::SIZE > 0);
        println!("ReferralBinding SIZE: {}", ReferralBinding::SIZE);
    }

    #[test]
    fn test_referral_binding_creation() {
        let referee = Pubkey::new_unique();
        let referrer = Pubkey::new_unique();
        let link = Pubkey::new_unique();
        
        let binding = ReferralBinding::new(referee, referrer, link, 254, 1700000000);
        
        assert_eq!(binding.referee, referee);
        assert_eq!(binding.referrer, referrer);
        assert_eq!(binding.referral_link, link);
        assert_eq!(binding.trade_count, 0);
    }

    #[test]
    fn test_referral_binding_trade_recording() {
        let referee = Pubkey::new_unique();
        let referrer = Pubkey::new_unique();
        let link = Pubkey::new_unique();
        
        let mut binding = ReferralBinding::new(referee, referrer, link, 254, 1700000000);
        
        // 记录第一笔交易
        binding.record_trade(1000_000_000, 18_000_000, 10_000_000, 1700001000);
        assert_eq!(binding.trade_count, 1);
        assert_eq!(binding.referee_volume_e6, 1000_000_000);
        assert_eq!(binding.referrer_rewards_e6, 18_000_000);
        assert_eq!(binding.referee_discounts_e6, 10_000_000);
        assert_eq!(binding.last_trade_ts, 1700001000);
        
        // 记录第二笔交易
        binding.record_trade(500_000_000, 9_000_000, 5_000_000, 1700002000);
        assert_eq!(binding.trade_count, 2);
        assert_eq!(binding.referee_volume_e6, 1500_000_000);
        assert_eq!(binding.referrer_rewards_e6, 27_000_000);
        assert_eq!(binding.referee_discounts_e6, 15_000_000);
    }
}

