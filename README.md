# 1024 Fund Program

LP 基金程序 - 用户基金管理、LP 投资、费用收取

## 概述

Fund Program 允许用户创建和管理投资基金，接受 LP 投资，并通过 1024 DEX 进行交易。

## 功能

### 基金管理
- **创建基金**: 设置名称、费用结构
- **管理配置**: 更新费用、开关存款
- **关闭基金**: 退还所有 LP 后关闭

### LP 投资
- **存入**: LP 存入 USDC，获得份额代币
- **赎回**: 销毁份额代币，取回 USDC
- **NAV 追踪**: 实时净值计算

### 费用机制
- **管理费**: 年化比例 (最高 10%)，按时间线性计提
- **业绩费**: 盈利比例 (最高 50%)，仅对超过 HWM 的盈利收取
- **High Water Mark**: 保护 LP 利益

## 账户结构

### FundConfig (全局配置)
```rust
pub struct FundConfig {
    pub authority: Pubkey,      // 管理员
    pub vault_program: Pubkey,  // Vault 程序
    pub ledger_program: Pubkey, // Ledger 程序
    pub total_funds: u64,       // 总基金数
    pub active_funds: u64,      // 活跃基金数
    pub is_paused: bool,        // 暂停状态
}
```

### Fund (基金账户)
```rust
pub struct Fund {
    pub manager: Pubkey,        // 基金经理
    pub name: [u8; 32],         // 基金名称
    pub fund_vault: Pubkey,     // USDC 存储
    pub share_mint: Pubkey,     // 份额代币
    pub fee_config: FeeConfig,  // 费用配置
    pub stats: FundStats,       // 统计数据
    pub is_open: bool,          // 开放存款
    pub is_paused: bool,        // 暂停状态
}
```

### LPPosition (LP 持仓)
```rust
pub struct LPPosition {
    pub fund: Pubkey,           // 所属基金
    pub investor: Pubkey,       // 投资者
    pub shares: u64,            // 持有份额
    pub deposit_nav_e6: i64,    // 存入时 NAV
    pub total_deposited_e6: i64, // 总存入
}
```

## 指令

| 指令 | 说明 | 调用者 |
|------|------|--------|
| Initialize | 初始化程序 | Admin |
| CreateFund | 创建基金 | Manager |
| DepositToFund | LP 存入 | LP |
| RedeemFromFund | LP 赎回 | LP |
| CollectFees | 收取费用 | Manager |
| TradeFund | 基金交易 | Manager |
| UpdateNAV | 更新 NAV | Anyone |

## NAV 计算

```
NAV = Total Value / Total Shares
Total Value = Deposits - Withdrawals + Realized PnL - Fees
```

## 费用计算

### 管理费 (Management Fee)
```
Fee = AUM × Fee_Rate × Time_Elapsed / Year
```

### 业绩费 (Performance Fee)
```
Fee = (NAV - HWM) × Total_Value × Fee_Rate / NAV
(仅当 NAV > HWM 时收取)
```

## PDA Seeds

| 账户 | Seeds |
|------|-------|
| FundConfig | `["fund_config"]` |
| Fund | `["fund", manager, fund_index]` |
| FundVault | `["fund_vault", fund]` |
| ShareMint | `["share_mint", fund]` |
| LPPosition | `["lp_position", fund, investor]` |

## 构建

```bash
cd 1024-fund-program
cargo build-sbf
```

## 测试

```bash
cargo test --lib
```

## 文件结构

```
src/
├── lib.rs          # 入口点
├── state.rs        # 账户结构
├── instruction.rs  # 指令定义
├── processor.rs    # 处理器
├── error.rs        # 错误类型
├── utils.rs        # 工具函数
└── cpi.rs          # CPI helpers
```

## License

MIT

