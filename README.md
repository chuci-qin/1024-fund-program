# 1024 Fund Program

> LP 基金程序 - 基金管理、保险基金、返佣系统、预测市场手续费

---

## 📋 目录

- [概述](#概述)
- [架构设计](#架构设计)
- [账户结构](#账户结构)
- [指令详解](#指令详解)
- [LP 投资管理](#lp-投资管理)
- [保险基金机制](#保险基金机制)
- [返佣系统](#返佣系统)
- [预测市场手续费](#预测市场手续费)
- [PDA 地址推导](#pda-地址推导)
- [构建与部署](#构建与部署)
- [测试](#测试)
- [错误代码](#错误代码)

---

## 概述

### 程序职责

1024 Fund Program 是 1024 DEX 生态系统的资金池管理核心，负责：

| 职责 | 说明 |
|------|------|
| **LP 基金管理** | 创建/管理投资基金，接受 LP 投资 |
| **NAV 追踪** | 实时净值计算，业绩费/管理费收取 |
| **保险基金** | 清算收入、穿仓覆盖、ADL 触发 |
| **返佣系统** | 邀请链接、VIP 加成、自动分佣 |
| **预测市场手续费** | 铸造/赎回/交易费收取与分配 |
| **Square 平台支付** | 知识付费、订阅、打赏结算 |

### 部署信息

| 网络 | Program ID |
|------|-----------|
| 1024Chain Testnet | `FundPMaFfYBn5z8Dxv95qKmS1rB6HQXC2rvyQ8F4kUL` |
| 1024Chain Mainnet | TBD |

### 系统定位

```
                    ┌─────────────────────────────────────────┐
                    │           1024-fund-program              │
                    │           (资金池管理)                    │
                    └─────────────────┬───────────────────────┘
                                      │
      ┌───────────────┬───────────────┼───────────────┬───────────────┐
      │               │               │               │               │
      ▼               ▼               ▼               ▼               ▼
┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐
│ LP Fund   │  │ Insurance │  │ Referral  │  │ PM Fee    │  │ Square    │
│ Management│  │ Fund      │  │ System    │  │ Mgmt      │  │ Payment   │
├───────────┤  ├───────────┤  ├───────────┤  ├───────────┤  ├───────────┤
│- Create   │  │- Liquidate│  │- Invite   │  │- Minting  │  │- Purchase │
│- Deposit  │  │- ADL      │  │- VIP tier │  │- Trading  │  │- Subscribe│
│- Redeem   │  │- Cover    │  │- Reward   │  │- Creator  │  │- Donate   │
│- Fee      │  │- Snapshot │  │- Discount │  │- Maker    │  │           │
└───────────┘  └───────────┘  └───────────┘  └───────────┘  └───────────┘
```

---

## 架构设计

### Relayer 与多签支持

Fund Program 支持 Multi-Relayer 机制，用于跨链操作：

```rust
// 最多支持 5 个授权 Relayer
pub const MAX_RELAYERS: usize = 5;

// 默认限额
pub const DEFAULT_SINGLE_TX_LIMIT_E6: i64 = 100_000_000_000;  // $100,000
pub const DEFAULT_DAILY_LIMIT_E6: i64 = 1_000_000_000_000;    // $1,000,000
```

### Relayer 限额机制

```rust
pub struct RelayerLimits {
    pub single_tx_limit_e6: i64,   // 单笔交易限额
    pub daily_limit_e6: i64,       // 每日限额
    pub daily_used_e6: i64,        // 今日已用
    pub last_reset_ts: i64,        // 上次重置时间 (UTC 0点自动重置)
}
```

---

## 账户结构

### 1. FundConfig (全局配置)

**PDA Seeds:** `["fund_config"]`

```rust
pub struct FundConfig {
    pub discriminator: u64,
    pub authority: Pubkey,                          // 管理员
    pub vault_program: Pubkey,                      // Vault Program ID
    pub ledger_program: Pubkey,                     // Ledger Program ID
    pub total_funds: u64,                           // 总基金数
    pub active_funds: u64,                          // 活跃基金数
    pub total_tvl_e6: i64,                          // 总 TVL (e6)
    pub is_paused: bool,
    pub bump: u8,
    
    // Multi-Relayer 支持
    pub authorized_relayers: [Pubkey; 5],           // 授权 Relayer 列表
    pub relayer_active: [bool; 5],                  // Relayer 激活状态
    pub active_relayer_count: u8,                   // 活跃 Relayer 数量
    pub relayer_limits: RelayerLimits,              // 全局限额配置
    pub reserved: [u8; 32],
}
```

### 2. Fund (基金账户)

**PDA Seeds:** `["fund", manager_pubkey, fund_index.to_le_bytes()]`

```rust
pub struct Fund {
    pub discriminator: u64,
    pub manager: Pubkey,                // 基金经理
    pub name: [u8; 32],                 // 基金名称
    pub bump: u8,
    pub fund_vault: Pubkey,             // USDC 存储账户
    pub share_mint: Pubkey,             // 份额代币 Mint
    pub fee_config: FeeConfig,          // 费用配置
    pub stats: FundStats,               // 统计数据
    pub is_open: bool,                  // 开放存款
    pub is_paused: bool,
    pub created_at: i64,
    pub last_update_ts: i64,
    pub fund_index: u64,                // 唯一索引
    pub reserved: [u8; 64],
}

pub struct FeeConfig {
    pub management_fee_bps: u32,        // 管理费 (基点, 200=2%)
    pub performance_fee_bps: u32,       // 业绩费 (基点, 2000=20%)
    pub use_high_water_mark: bool,      // 使用高水位线
    pub fee_collection_interval: i64,   // 收费间隔 (秒)
}

pub struct FundStats {
    pub total_deposits_e6: i64,
    pub total_withdrawals_e6: i64,
    pub current_nav_e6: i64,            // 当前 NAV (1.0 = 1_000_000)
    pub high_water_mark_e6: i64,        // 高水位线
    pub total_management_fee_e6: i64,
    pub total_performance_fee_e6: i64,
    pub total_shares: u64,
    pub last_fee_collection_ts: i64,
    pub total_realized_pnl_e6: i64,
    pub lp_count: u32,
}
```

### 3. LPPosition (LP 持仓)

**PDA Seeds:** `["lp_position", fund_pubkey, investor_pubkey]`

```rust
pub struct LPPosition {
    pub discriminator: u64,
    pub fund: Pubkey,
    pub investor: Pubkey,
    pub shares: u64,                    // 持有份额
    pub deposit_nav_e6: i64,            // 存入时 NAV
    pub total_deposited_e6: i64,
    pub total_withdrawn_e6: i64,
    pub deposited_at: i64,
    pub last_update_ts: i64,
    pub bump: u8,
    pub reserved: [u8; 32],
}
```

### 4. InsuranceFundConfig (保险基金配置)

**PDA Seeds:** `["insurance_fund_config"]`

```rust
pub struct InsuranceFundConfig {
    pub discriminator: u64,
    pub fund: Pubkey,                             // 关联的 Fund 账户
    pub bump: u8,
    
    // 收入统计
    pub total_liquidation_income_e6: i64,         // 清算收入
    pub total_adl_profit_e6: i64,                 // ADL 盈余
    
    // 支出统计
    pub total_shortfall_payout_e6: i64,           // 穿仓支出
    
    // ADL 配置
    pub adl_trigger_threshold_e6: i64,            // ADL 触发阈值
    pub adl_trigger_count: u64,                   // ADL 触发次数
    
    // 1小时快照 (30%下降检测)
    pub balance_1h_ago_e6: i64,
    pub last_snapshot_ts: i64,
    
    // LP 赎回控制
    pub withdrawal_delay_secs: i64,               // 赎回延迟
    pub is_adl_in_progress: bool,                 // ADL 进行中
    
    pub authorized_caller: Pubkey,                // 授权调用方 (Ledger)
    pub last_update_ts: i64,
    pub reserved: [u8; 64],
}
```

### 5. ReferralConfig / ReferralLink / ReferralBinding (返佣系统)

```rust
pub struct ReferralConfig {
    pub discriminator: u64,
    pub authority: Pubkey,
    pub vault_program: Pubkey,
    pub referrer_share_bps: u16,              // 邀请人分成 (2000=20%)
    pub referee_discount_bps: u16,            // 被邀请人折扣 (1000=10%)
    pub referrer_vip_bonus_bps: [u16; 6],     // VIP 等级加成
    pub referee_vip_bonus_bps: [u16; 6],      // VIP 等级折扣加成
    pub min_settlement_amount_e6: i64,        // 最低结算金额
    pub reward_validity_secs: i64,            // 返佣有效期 (0=永久)
    
    // 统计
    pub total_rewards_paid_e6: i64,
    pub total_discounts_given_e6: i64,
    pub total_referral_links: u64,
    pub total_referred_users: u64,
    pub total_referred_volume_e6: i64,
    
    pub is_paused: bool,
    pub bump: u8,
    pub last_update_ts: i64,
    pub reserved: [u8; 64],
}

pub struct ReferralLink {
    pub discriminator: u64,
    pub referrer: Pubkey,
    pub code: [u8; 12],                       // 邀请码
    pub created_at: i64,
    pub is_active: bool,
    pub custom_referrer_share_bps: u16,       // 自定义分成
    pub custom_referee_discount_bps: u16,     // 自定义折扣
    pub referred_count: u32,
    pub total_volume_e6: i64,
    pub total_rewards_earned_e6: i64,
    pub total_discounts_given_e6: i64,
    pub bump: u8,
    pub reserved: [u8; 32],
}

pub struct ReferralBinding {
    pub discriminator: u64,
    pub referee: Pubkey,                      // 被邀请人
    pub referrer: Pubkey,                     // 邀请人
    pub referral_link: Pubkey,
    pub bound_at: i64,
    pub referee_volume_e6: i64,
    pub referrer_rewards_e6: i64,
    pub referee_discounts_e6: i64,
    pub trade_count: u64,
    pub last_trade_ts: i64,
    pub bump: u8,
    pub reserved: [u8; 32],
}
```

### 6. PredictionMarketFeeConfig (预测市场手续费配置)

**PDA Seeds:** `["prediction_market_fee_config"]`

```rust
pub struct PredictionMarketFeeConfig {
    pub discriminator: u64,
    pub prediction_market_fee_vault: Pubkey,      // USDC 手续费池
    pub bump: u8,
    
    // 费率配置 (basis points)
    pub prediction_market_minting_fee_bps: u16,        // 铸造费 (10=0.1%)
    pub prediction_market_redemption_fee_bps: u16,     // 赎回费
    pub prediction_market_trading_fee_taker_bps: u16,  // Taker 交易费
    pub prediction_market_trading_fee_maker_bps: u16,  // Maker 交易费 (通常 0)
    pub prediction_market_settlement_fee_bps: u16,     // 结算费
    
    // 分配比例 (总计 10000)
    pub prediction_market_protocol_share_bps: u16,     // 协议收入 (7000=70%)
    pub prediction_market_maker_reward_share_bps: u16, // 做市商奖励 (2000=20%)
    pub prediction_market_creator_share_bps: u16,      // 创建者分成 (1000=10%)
    
    // 累计统计
    pub prediction_market_total_minting_fee_e6: i64,
    pub prediction_market_total_redemption_fee_e6: i64,
    pub prediction_market_total_trading_fee_e6: i64,
    pub prediction_market_total_maker_rewards_e6: i64,
    pub prediction_market_total_creator_rewards_e6: i64,
    pub prediction_market_total_protocol_income_e6: i64,
    
    pub prediction_market_authorized_caller: Pubkey,   // PM Program
    pub authority: Pubkey,
    pub is_paused: bool,
    pub last_update_ts: i64,
    pub reserved: [u8; 64],
}
```

---

## 指令详解

### 初始化指令

| 指令 | 说明 |
|------|------|
| `Initialize` | 初始化 FundConfig |
| `InitializeInsuranceFund` | 初始化保险基金 |
| `InitializeReferral` | 初始化返佣系统 |
| `InitializePredictionMarketFeeConfig` | 初始化 PM 手续费配置 |

### LP 基金指令

| 指令 | 说明 | 调用者 |
|------|------|--------|
| `CreateFund` | 创建新基金 | 基金经理 |
| `DepositToFund` | LP 存入 | LP |
| `RedeemFromFund` | LP 赎回 | LP |
| `CollectFees` | 收取费用 | 基金经理 |
| `TradeFund` | 基金交易 | 基金经理 |
| `UpdateNAV` | 更新净值 | 任何人 |
| `SetFundOpen` | 开关存款 | 基金经理 |
| `CloseFund` | 关闭基金 | 基金经理 |

### 保险基金指令 (CPI)

| 指令 | 说明 | 调用者 |
|------|------|--------|
| `AddLiquidationIncome` | 添加清算收入 | Ledger |
| `AddADLProfit` | 添加 ADL 盈余 | Ledger |
| `CoverShortfall` | 覆盖穿仓 | Ledger |
| `AddTradingFee` | 添加交易手续费 | Ledger |
| `SetADLInProgress` | 设置 ADL 状态 | Ledger |
| `UpdateHourlySnapshot` | 更新小时快照 | Relayer |
| `CheckADLTrigger` | 检查 ADL 条件 | 任何人 |
| `RedeemFromInsuranceFund` | 保险基金赎回 | LP |

### 返佣系统指令

| 指令 | 说明 | 调用者 |
|------|------|--------|
| `CreateReferralLink` | 创建邀请链接 | 用户 |
| `BindReferral` | 绑定邀请关系 | 新用户 |
| `RecordReferralTrade` | 记录返佣交易 | Ledger (CPI) |
| `UpdateReferralConfig` | 更新返佣配置 | Admin |
| `DeactivateReferralLink` | 停用邀请链接 | 邀请人 |
| `SetCustomReferralRates` | 设置自定义比例 | Admin |

### 预测市场手续费指令

| 指令 | 说明 | 调用者 |
|------|------|--------|
| `CollectPredictionMarketMintingFee` | 收取铸造费 | PM Program (CPI) |
| `CollectPredictionMarketRedemptionFee` | 收取赎回费 | PM Program (CPI) |
| `CollectPredictionMarketTradingFee` | 收取交易费 | PM Program (CPI) |
| `DistributePredictionMarketMakerReward` | 发放做市商奖励 | Admin / PM |
| `DistributePredictionMarketCreatorReward` | 发放创建者分成 | PM Program (CPI) |
| `UpdatePredictionMarketFeeConfig` | 更新费率配置 | Admin |
| `SetPredictionMarketFeePaused` | 暂停/恢复 | Admin |

### Relayer 指令

| 指令 | 说明 |
|------|------|
| `RelayerDepositToFund` | Relayer 代理存款 |
| `RelayerRedeemFromFund` | Relayer 代理赎回 |
| `RelayerRedeemFromInsuranceFund` | Relayer 代理保险基金赎回 |
| `RelayerSquarePayment` | Relayer 代理 Square 支付 |
| `RelayerBindReferral` | Relayer 代理绑定邀请 |
| `AddRelayer` | 添加 Relayer (Admin) |
| `RemoveRelayer` | 移除 Relayer (Admin) |
| `UpdateRelayerLimits` | 更新 Relayer 限额 (Admin) |

---

## LP 投资管理

### NAV 计算

```rust
// NAV = Total Value / Total Shares
// Total Value = Deposits - Withdrawals + Realized PnL - Fees

impl FundStats {
    pub fn total_value_e6(&self) -> i64 {
        self.total_deposits_e6
            .saturating_sub(self.total_withdrawals_e6)
            .saturating_add(self.total_realized_pnl_e6)
            .saturating_sub(self.total_management_fee_e6)
            .saturating_sub(self.total_performance_fee_e6)
    }
}
```

### 费用计算

**管理费 (时间线性):**
```
Management Fee = AUM × Fee_Rate × Time_Elapsed / Year
```

**业绩费 (High Water Mark):**
```
// 仅当 NAV > HWM 时收取
Performance Fee = (NAV - HWM) × Total_Value × Fee_Rate / NAV
```

---

## 保险基金机制

### ADL 三重触发条件

```rust
pub fn should_trigger_adl(&self, balance: i64, shortfall: i64) -> ADLTriggerReason {
    // 条件 1: 穿仓触发 - 保险基金无法覆盖
    if shortfall > 0 && balance < shortfall {
        return ADLTriggerReason::Bankruptcy;
    }
    
    // 条件 2: 余额不足 - 低于阈值
    if balance < self.adl_trigger_threshold_e6 {
        return ADLTriggerReason::InsufficientBalance;
    }
    
    // 条件 3: 1小时快速下降 - 下降超过 30%
    if self.balance_1h_ago_e6 > 0 {
        let threshold = self.balance_1h_ago_e6 * 70 / 100;
        if balance < threshold {
            return ADLTriggerReason::RapidDecline;
        }
    }
    
    ADLTriggerReason::None
}
```

### 保险基金流程

```
┌─────────────────────────────────────────────────────────────────┐
│                      保险基金资金流                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌───────────────┐     收入来源                                │
│   │               │ ◄── 清算罚金 (Liquidation)                  │
│   │   Insurance   │ ◄── ADL 盈余                                │
│   │     Fund      │ ◄── 交易手续费 (10% 分成)                    │
│   │               │                                             │
│   └───────┬───────┘                                             │
│           │                                                     │
│           │  支出                                                │
│           ▼                                                     │
│   ┌───────────────┐                                             │
│   │ Cover Shortfall│ ──► 穿仓覆盖                                │
│   └───────────────┘                                             │
│           │                                                     │
│           │ 不足时                                               │
│           ▼                                                     │
│   ┌───────────────┐                                             │
│   │  Trigger ADL  │ ──► 自动减仓盈利方                           │
│   └───────────────┘                                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 返佣系统

### VIP 等级加成

| VIP 等级 | 邀请人加成 | 被邀请人折扣加成 |
|----------|-----------|------------------|
| VIP 0 | 0% | 0% |
| VIP 1 | 2% | 2% |
| VIP 2 | 5% | 5% |
| VIP 3 | 10% | 10% |
| VIP 4 | 15% | 15% |
| VIP 5 | 20% | 20% |

### 返佣计算示例

```rust
// 基础配置: 邀请人 20%, 被邀请人折扣 10%
// 交易手续费: $100

// VIP 0:
// 被邀请人折扣: $100 × 10% = $10
// 实际收费: $100 - $10 = $90
// 邀请人返佣: $90 × 20% = $18
// 平台收入: $90 - $18 = $72

// VIP 3 (取两人最高等级):
// 被邀请人折扣: $100 × 20% = $20
// 实际收费: $100 - $20 = $80
// 邀请人返佣: $80 × 30% = $24
// 平台收入: $80 - $24 = $56
```

---

## 预测市场手续费

### 费率配置

| 费用类型 | 默认费率 | 说明 |
|----------|---------|------|
| 铸造费 | 0.1% | MintCompleteSet 时收取 |
| 赎回费 | 0.1% | RedeemCompleteSet 时收取 |
| Taker 交易费 | 0.1% | 吃单交易 |
| Maker 交易费 | 0% | 挂单交易 |

### 分配比例

| 接收方 | 默认比例 | 说明 |
|--------|---------|------|
| 协议收入 | 70% | 进入协议金库 |
| 做市商奖励 | 20% | 激励提供流动性 |
| 创建者分成 | 10% | 市场创建者收益 |

---

## PDA 地址推导

### TypeScript 示例

```typescript
const FUND_PROGRAM_ID = new PublicKey('FundPMaFfYBn5z8Dxv95qKmS1rB6HQXC2rvyQ8F4kUL');

// FundConfig PDA
const [fundConfigPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("fund_config")],
    FUND_PROGRAM_ID
);

// Fund PDA
const fundIndex = 0n;
const [fundPDA] = await PublicKey.findProgramAddress(
    [
        Buffer.from("fund"),
        manager.toBuffer(),
        Buffer.from(fundIndex.toString(16).padStart(16, '0'), 'hex'),
    ],
    FUND_PROGRAM_ID
);

// Fund Vault PDA
const [fundVaultPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("fund_vault"), fundPDA.toBuffer()],
    FUND_PROGRAM_ID
);

// Share Mint PDA
const [shareMintPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("share_mint"), fundPDA.toBuffer()],
    FUND_PROGRAM_ID
);

// LP Position PDA
const [lpPositionPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("lp_position"), fundPDA.toBuffer(), investor.toBuffer()],
    FUND_PROGRAM_ID
);

// Insurance Fund Config PDA
const [insuranceConfigPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("insurance_fund_config")],
    FUND_PROGRAM_ID
);

// Referral Config PDA
const [referralConfigPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("referral_config")],
    FUND_PROGRAM_ID
);

// Referral Link PDA
const [referralLinkPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("referral_link"), referrer.toBuffer()],
    FUND_PROGRAM_ID
);

// Referral Binding PDA
const [referralBindingPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("referral_binding"), referee.toBuffer()],
    FUND_PROGRAM_ID
);

// Prediction Market Fee Config PDA
const [pmFeeConfigPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("prediction_market_fee_config")],
    FUND_PROGRAM_ID
);
```

---

## 构建与部署

### 构建

```bash
cd 1024-fund-program

# 编译检查
cargo check

# 运行测试
cargo test --lib

# 构建 BPF 程序
cargo build-sbf
```

### 部署

```bash
# 部署到 1024Chain Testnet
solana program deploy target/deploy/fund_program.so \
    --url https://testnet-rpc.1024chain.com/rpc/ \
    --program-id FundPMaFfYBn5z8Dxv95qKmS1rB6HQXC2rvyQ8F4kUL \
    --use-rpc
```

---

## 测试

### 单元测试覆盖

| 测试项 | 文件 | 状态 |
|--------|------|------|
| FundConfig SIZE 计算 | `state.rs` | ✅ |
| Fund 创建和存取款 | `state.rs` | ✅ |
| LPPosition 收益计算 | `state.rs` | ✅ |
| FundStats NAV 更新 | `state.rs` | ✅ |
| InsuranceFundConfig ADL 触发 | `state.rs` | ✅ |
| InsuranceFundConfig 覆盖穿仓 | `state.rs` | ✅ |
| SquarePaymentRecord 创建 | `state.rs` | ✅ |
| ReferralConfig VIP 加成 | `state.rs` | ✅ |
| ReferralConfig 返佣计算 | `state.rs` | ✅ |
| ReferralLink 统计 | `state.rs` | ✅ |
| ReferralBinding 交易记录 | `state.rs` | ✅ |
| 指令序列化 | `instruction.rs` | ✅ |

### 运行测试

```bash
cargo test --lib
# 20+ tests passed
```

---

## 错误代码

| 错误 | Code | 说明 |
|------|------|------|
| `InsufficientShares` | 0 | 份额不足 |
| `InsufficientBalance` | 1 | 余额不足 |
| `FundNotOpen` | 2 | 基金未开放存款 |
| `FundPaused` | 3 | 基金已暂停 |
| `InvalidManager` | 4 | 非基金经理 |
| `InvalidFeeConfig` | 5 | 无效的费用配置 |
| `NAVCalculationError` | 6 | NAV 计算错误 |
| `Overflow` | 7 | 数值溢出 |
| `ADLInProgress` | 8 | ADL 进行中，禁止赎回 |
| `WithdrawalDelayNotMet` | 9 | 赎回延迟未满足 |
| `UnauthorizedCaller` | 10 | 未授权的 CPI 调用 |
| `ReferralLinkNotActive` | 11 | 邀请链接未激活 |
| `AlreadyBound` | 12 | 已绑定邀请关系 |
| `PMFeePaused` | 13 | 预测市场手续费已暂停 |
| `RelayerLimitExceeded` | 14 | Relayer 限额超出 |
| `MaxRelayersReached` | 15 | Relayer 数量已满 |
| `RelayerNotFound` | 16 | Relayer 未找到 |

---

## 文件结构

```
1024-fund-program/
├── Cargo.toml
├── README.md
├── rust-toolchain.toml
└── src/
    ├── lib.rs          # 程序入口点
    ├── state.rs        # 账户结构定义
    ├── instruction.rs  # 指令枚举定义
    ├── processor.rs    # 指令处理逻辑
    ├── error.rs        # 错误类型
    ├── utils.rs        # 工具函数 (NAV/Fee 计算)
    └── cpi.rs          # CPI Helper 函数
```

---

## License

MIT

---

*Last Updated: 2025-12-09*
