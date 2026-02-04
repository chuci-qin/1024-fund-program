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

### 6. PerpTradingFeeConfig (Perp 交易手续费配置) 🆕

**PDA Seeds:** `["perp_trading_fee_config"]`

> Perp 永续合约交易手续费管理

```rust
pub struct PerpTradingFeeConfig {
    pub discriminator: u64,
    pub perp_fee_vault: Pubkey,              // Perp 手续费池 (USDC)
    pub bump: u8,
    
    // === 默认费率配置 (basis points) ===
    pub default_taker_fee_bps: u16,          // 默认 Taker 费率 (50 = 0.05%)
    pub default_maker_fee_bps: u16,          // 默认 Maker 费率 (20 = 0.02%)
    
    // === 分配比例 (总计 10000) ===
    pub protocol_share_bps: u16,             // 协议收入 (6000 = 60%)
    pub insurance_fund_share_bps: u16,       // 保险基金 (2000 = 20%)
    pub referral_share_bps: u16,             // 返佣系统 (1500 = 15%)
    pub maker_reward_share_bps: u16,         // 做市商奖励 (500 = 5%)
    
    // === 累计统计 ===
    pub total_taker_fee_collected_e6: i64,   // Taker 费用总收入
    pub total_maker_fee_collected_e6: i64,   // Maker 费用总收入
    pub total_protocol_income_e6: i64,       // 协议净收入
    pub total_insurance_fund_income_e6: i64, // 保险基金收入
    pub total_referral_rewards_e6: i64,      // 返佣系统收入
    pub total_maker_rewards_e6: i64,         // 做市商奖励收入
    
    // === 管理 ===
    pub authorized_caller: Pubkey,           // Ledger Program
    pub authority: Pubkey,
    pub is_paused: bool,
    pub last_update_ts: i64,
    pub reserved: [u8; 64],
}
```

### Perp 手续费流程

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Perp 手续费完整流程                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   Step 1: 成交时收取                                                     │
│   ├── Taker 支付 0.05% taker_fee                                        │
│   ├── Maker 支付 0.02% maker_fee                                        │
│   └── 总手续费 = taker_fee + maker_fee = 0.07%                          │
│                                                                         │
│   Step 2: 即时分配                                                       │
│   ├── 协议收入 (60%) → 协议国库 Vault                                   │
│   ├── 保险基金 (20%) → Insurance Fund                                   │
│   ├── 返佣系统 (15%) → 邀请人返佣池                                     │
│   └── 做市商奖励 (5%) → 做市商奖励池                                     │
│                                                                         │
│   Step 3: 返佣处理                                                       │
│   ├── 检查 Taker 是否有 ReferralBinding                                 │
│   ├── 有绑定: 邀请人获得 referral_pool × referrer_share                 │
│   │          被邀请人已在 taker_fee 中获得折扣                          │
│   └── 无绑定: 返佣部分回流协议                                          │
│                                                                         │
│   Step 4: 做市商奖励 (每日结算)                                          │
│   ├── 累计每日做市商贡献 (成交量/深度/在线时长)                         │
│   ├── 按权重分配做市商奖励池                                            │
│   └── 发放到做市商 Vault UserAccount                                    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### 7. PredictionMarketFeeConfig (预测市场手续费配置)

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

### 8. SpotTradingFeeConfig (Spot 交易手续费配置) 🆕

**PDA Seeds:** `["spot_trading_fee_config"]`

> Spot 交易手续费管理，类似于 PredictionMarketFeeConfig

```rust
pub struct SpotTradingFeeConfig {
    pub discriminator: u64,
    pub spot_fee_vault: Pubkey,              // 多 Token 手续费池 (可使用 USDC 统一或分 Token)
    pub bump: u8,
    
    // === 默认费率配置 (basis points) ===
    // 注意: 每个 SpotMarket 可覆盖默认费率
    pub default_taker_fee_bps: u16,          // 默认 Taker 费率 (50 = 0.05%)
    pub default_maker_fee_bps: i16,          // 默认 Maker 费率 (-20 = -0.02% 返佣)
    
    // === 分配比例 (总计 10000) ===
    pub protocol_share_bps: u16,             // 协议收入 (6000 = 60%)
    pub insurance_fund_share_bps: u16,       // 保险基金 (2000 = 20%)
    pub referral_share_bps: u16,             // 返佣系统 (1500 = 15%)
    pub maker_reward_share_bps: u16,         // 做市商奖励 (500 = 5%)
    
    // === 累计统计 ===
    pub total_taker_fee_collected_e6: i64,   // Taker 费用总收入
    pub total_maker_fee_paid_e6: i64,        // Maker 返佣总支出 (负数)
    pub total_protocol_income_e6: i64,       // 协议净收入
    pub total_insurance_fund_income_e6: i64, // 保险基金收入
    pub total_referral_rewards_e6: i64,      // 返佣系统收入
    pub total_maker_rewards_e6: i64,         // 做市商奖励收入
    
    // === 按 Token 统计 (可选，用于多 Token 手续费) ===
    pub fee_by_token: [TokenFeeStats; 16],   // 按 Token 分别统计
    
    // === 管理 ===
    pub authorized_caller: Pubkey,           // Vault Program PDA
    pub authority: Pubkey,
    pub is_paused: bool,
    pub last_update_ts: i64,
    pub reserved: [u8; 64],
}

/// 按 Token 的手续费统计
pub struct TokenFeeStats {
    pub token_index: u16,                    // Token 索引
    pub total_collected_e6: i64,             // 该 Token 收取的总手续费
    pub total_distributed_e6: i64,           // 该 Token 分配的总手续费
}
```

### Spot 手续费流程

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Spot 手续费完整流程                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   Step 1: 成交时收取                                                     │
│   ├── Taker 支付 taker_fee (从 Quote Token 扣除)                        │
│   ├── Maker 收取 maker_rebate (负费率 = 返佣)                           │
│   └── 净手续费 = taker_fee - maker_rebate                               │
│                                                                         │
│   Step 2: 即时分配                                                       │
│   ├── 协议收入 (60%) → 协议国库 Vault                                   │
│   ├── 保险基金 (20%) → Insurance Fund                                   │
│   ├── 返佣系统 (15%) → 邀请人返佣池                                     │
│   └── 做市商奖励 (5%) → 做市商奖励池                                     │
│                                                                         │
│   Step 3: 返佣处理                                                       │
│   ├── 检查 ReferralBinding 是否存在                                     │
│   ├── 计算邀请人返佣 = 返佣池 × referrer_share                          │
│   ├── 计算被邀请人折扣 = 已包含在 taker_fee 中                          │
│   └── 更新 ReferralBinding 统计                                         │
│                                                                         │
│   Step 4: 做市商奖励 (每日/每周结算)                                     │
│   ├── 统计做市商提供的流动性和成交量                                    │
│   ├── 按权重分配做市商奖励池                                            │
│   └── 发放到做市商账户                                                  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Spot 手续费 vs Perp/PM 手续费对比

| 维度 | Perp 手续费 | Spot 手续费 | PM 手续费 |
|------|------------|------------|----------|
| 结算币种 | USDC | Quote Token (通常 USDC) | USDC |
| Taker 默认费率 | 0.05% | 0.05% | 0.1% |
| Maker 默认费率 | 0.02% | -0.02% (返佣) | 0% |
| 协议分成 | 60% | 60% | 70% |
| 保险基金 | 20% | 20% | - |
| 返佣系统 | 15% | 15% | - |
| 做市商奖励 | 5% | 5% | 20% |
| 创建者分成 | - | - | 10% |

---

## 指令详解

### 初始化指令

| 指令 | 说明 |
|------|------|
| `Initialize` | 初始化 FundConfig |
| `InitializeInsuranceFund` | 初始化保险基金 |
| `InitializeReferral` | 初始化返佣系统 |
| `InitializePerpTradingFeeConfig` | 初始化 Perp 手续费配置 🆕 |
| `InitializeSpotTradingFeeConfig` | 初始化 Spot 手续费配置 🆕 |
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

### Perp 交易手续费指令 🆕

| 指令 | 说明 | 调用者 |
|------|------|--------|
| `InitializePerpTradingFeeConfig` | 初始化 Perp 手续费配置 | Admin |
| `CollectPerpTradingFee` | 收取 Perp 交易手续费 | Ledger Program (CPI) |
| `DistributePerpFee` | 分配 Perp 手续费到各池 | Relayer / System |
| `DistributePerpMakerReward` | 发放 Perp 做市商奖励 | Admin / Relayer |
| `UpdatePerpTradingFeeConfig` | 更新 Perp 费率配置 | Admin |
| `SetPerpFeePaused` | 暂停/恢复 Perp 手续费 | Admin |

### Spot 交易手续费指令 🆕

| 指令 | 说明 | 调用者 |
|------|------|--------|
| `InitializeSpotTradingFeeConfig` | 初始化 Spot 手续费配置 | Admin |
| `CollectSpotTradingFee` | 收取 Spot 交易手续费 | Vault Program (CPI) |
| `DistributeSpotFee` | 分配 Spot 手续费到各池 | Relayer / System |
| `DistributeSpotMakerReward` | 发放 Spot 做市商奖励 | Admin / Relayer |
| `UpdateSpotTradingFeeConfig` | 更新 Spot 费率配置 | Admin |
| `SetSpotFeePaused` | 暂停/恢复 Spot 手续费 | Admin |
| `GetSpotFeeStats` | 查询 Spot 手续费统计 | Anyone (Read) |

### 做市商奖励指令 🆕

| 指令 | 说明 | 调用者 |
|------|------|--------|
| `RegisterAsMaker` | 注册为做市商 | 用户 |
| `UpdateMakerStatus` | 更新做市商状态 | Admin |
| `CalculateMakerRewards` | 计算 Epoch 做市商奖励 | Relayer |
| `ClaimMakerReward` | 领取做市商奖励 | 做市商 |
| `GetMakerStats` | 查询做市商统计 | Anyone (Read) |

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

## 做市商奖励机制 🆕

### 设计目标

激励做市商提供深度流动性，确保 DEX 和预测市场的交易体验接近 CEX。

### 奖励来源

| 来源 | 分配比例 | 说明 |
|------|---------|------|
| Perp 手续费 | 5% | 进入 Perp Maker Reward Pool |
| Spot 手续费 | 5% | 进入 Spot Maker Reward Pool |
| PM 手续费 | 20% | 进入 PM Maker Reward Pool |

### 贡献权重计算

每个 Epoch (24 小时) 结算一次，做市商权重由三个因素决定：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    做市商权重计算公式                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   final_weight = volume_weight × 0.4                                    │
│                + depth_weight × 0.4                                     │
│                + uptime_weight × 0.2                                    │
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────────┐  │
│   │ 成交量权重 (Volume Weight) - 40%                                 │  │
│   ├─────────────────────────────────────────────────────────────────┤  │
│   │ volume_weight = maker_filled_volume / total_maker_filled_volume │  │
│   │                                                                 │  │
│   │ 说明: 做市商的成交量占总成交量的比例                             │  │
│   └─────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────────┐  │
│   │ 流动性深度权重 (Depth Weight) - 40%                              │  │
│   ├─────────────────────────────────────────────────────────────────┤  │
│   │ depth_weight = avg_depth_in_bbo / total_avg_depth               │  │
│   │                                                                 │  │
│   │ 说明: 在 BBO (Best Bid/Offer) ±1% 范围内的平均挂单量占比        │  │
│   │       鼓励在接近市价的位置提供流动性                             │  │
│   └─────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────────┐  │
│   │ 在线时长权重 (Uptime Weight) - 20%                               │  │
│   ├─────────────────────────────────────────────────────────────────┤  │
│   │ uptime_weight = active_quoting_time / epoch_duration            │  │
│   │                                                                 │  │
│   │ 说明: 做市商订单存在时间 / Epoch 总时长                          │  │
│   │       鼓励持续提供流动性                                         │  │
│   └─────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 奖励分配流程

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    做市商奖励分配流程                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   每日 UTC 00:00 由 Relayer 自动触发:                                   │
│                                                                         │
│   Step 1: 收集上一 Epoch 数据                                           │
│   ├── 从 Ledger 获取所有 Maker 成交记录                                 │
│   ├── 从订单簿快照获取深度数据                                          │
│   └── 从订单日志获取在线时长数据                                        │
│                                                                         │
│   Step 2: 计算各做市商权重                                              │
│   ├── 计算 volume_weight                                                │
│   ├── 计算 depth_weight                                                 │
│   └── 计算 uptime_weight                                                │
│                                                                         │
│   Step 3: 分配奖励                                                      │
│   ├── maker_A_reward = pool_total × (A_weight / Σ weights)              │
│   └── 写入 MakerRewardRecord PDA                                        │
│                                                                         │
│   Step 4: 做市商领取                                                    │
│   ├── 做市商调用 ClaimMakerReward                                       │
│   └── 奖励直接存入 Vault UserAccount.available_balance                  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 做市商资格要求

| 要求 | 阈值 | 说明 |
|------|------|------|
| 最低质押 | 1,000 USDC | 作为行为保证金 |
| 最低日成交量 | $10,000 | 需要实际产生交易 |
| 最低在线时长 | 80% | 每日 19.2 小时 |
| KYC | 可选 | 高等级做市商需要 |

### 违规处罚

| 违规行为 | 处罚 |
|---------|------|
| 连续 3 天不满足最低要求 | 警告通知 |
| 连续 7 天不满足最低要求 | 暂停做市商资格 |
| 恶意操纵价格 | 没收保证金 + 永久禁止 |
| 洗盘交易 | 没收该 Epoch 奖励 + 警告 |

### MakerRewardRecord 账户结构

**PDA Seeds:** `["maker_reward", market_type, epoch.to_le_bytes()]`

```rust
pub struct MakerRewardRecord {
    pub discriminator: u64,
    pub market_type: MarketType,         // Perp / Spot / PM
    pub epoch: u64,                      // Epoch 编号 (天数)
    pub epoch_start_ts: i64,             // Epoch 开始时间
    pub epoch_end_ts: i64,               // Epoch 结束时间
    
    // 奖励池
    pub total_pool_e6: i64,              // 该 Epoch 奖励池总额
    pub distributed_e6: i64,             // 已分配金额
    
    // 做市商列表 (最多 100 个)
    pub maker_count: u16,
    pub makers: [MakerRewardEntry; 100],
    
    pub is_finalized: bool,              // 是否已完成分配
    pub created_at: i64,
    pub bump: u8,
}

pub struct MakerRewardEntry {
    pub maker: Pubkey,                   // 做市商地址
    pub volume_weight_e6: u32,           // 成交量权重 (e6)
    pub depth_weight_e6: u32,            // 深度权重 (e6)
    pub uptime_weight_e6: u32,           // 在线时长权重 (e6)
    pub final_weight_e6: u32,            // 最终权重 (e6)
    pub reward_e6: i64,                  // 分配的奖励 (e6)
    pub claimed: bool,                   // 是否已领取
    pub claimed_at: i64,                 // 领取时间
}

pub enum MarketType {
    Perp = 0,
    Spot = 1,
    PredictionMarket = 2,
}
```

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

// Perp Trading Fee Config PDA
const [perpFeeConfigPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("perp_trading_fee_config")],
    FUND_PROGRAM_ID
);

// Spot Trading Fee Config PDA
const [spotFeeConfigPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("spot_trading_fee_config")],
    FUND_PROGRAM_ID
);

// Prediction Market Fee Config PDA
const [pmFeeConfigPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("prediction_market_fee_config")],
    FUND_PROGRAM_ID
);

// Maker Reward Record PDA (for specific epoch)
const marketType = 0; // 0=Perp, 1=Spot, 2=PM
const epoch = 1234n;
const epochBuffer = Buffer.alloc(8);
epochBuffer.writeBigUInt64LE(epoch);
const [makerRewardPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("maker_reward"), Buffer.from([marketType]), epochBuffer],
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

*Last Updated: 2025-01-28*

---

## 更新日志

### 2025-01-28
- 添加 `PerpTradingFeeConfig` 结构和相关指令
- 完善 `SpotTradingFeeConfig` 文档
- 添加完整的做市商奖励机制说明
- 添加 `MakerRewardRecord` 账户结构
- 更新初始化指令列表

### 2025-12-09
- 初始版本
