# 1024 Fund Program Scripts

管理 Fund Program 配置的脚本集合，包括预测市场手续费配置。

## 安装依赖

```bash
cd scripts
npm install
```

## 环境变量

可以通过环境变量自定义 RPC 和密钥路径：

```bash
export RPC_URL="https://testnet-rpc.1024chain.com/rpc/"
export KEYPAIR_PATH="$HOME/1024chain-testnet/keys/faucet.json"
```

## 脚本列表

### 1. 一键部署 PM Fee 系统

```bash
./deploy_pm_fee.sh
```

这将执行完整的部署流程：初始化配置 → 验证配置。

### 2. 初始化 PM Fee Config

```bash
node init_pm_fee_config.js
```

首次部署时运行，创建 `PredictionMarketFeeConfig` PDA。

### 3. 查询当前配置

```bash
node query_pm_fee_config.js
```

显示当前的费率配置和分配比例。

### 4. 更新费率配置

```bash
# 更新单个费率
node update_pm_fee_config.js --minting-fee 20  # 0.2%

# 更新多个费率
node update_pm_fee_config.js --minting-fee 20 --taker-fee 15

# 更新分配比例
node update_pm_fee_config.js --protocol-share 6000 --maker-share 3000 --creator-share 1000
```

### 5. 暂停/恢复费用收取

```bash
# 暂停（紧急情况）
node set_pm_fee_paused.js --paused true

# 恢复
node set_pm_fee_paused.js --paused false
```

## 默认费率配置

| 类型 | 默认值 | 说明 |
|------|--------|------|
| 铸造费 | 0.1% (10 bps) | Complete Set 铸造时收取 |
| 赎回费 | 0.1% (10 bps) | Complete Set 赎回时收取 |
| Taker 交易费 | 0.1% (10 bps) | 吃单方交易时收取 |
| Maker 交易费 | 0% (0 bps) | 挂单方不收费（激励做市） |

## 费用分配比例

| 接收方 | 默认比例 | 说明 |
|--------|---------|------|
| 协议 | 70% | 平台收入 |
| Maker | 20% | 做市商奖励 |
| Creator | 10% | 市场创建者分成 |

## npm 脚本

```bash
npm run deploy   # 一键部署
npm run init     # 初始化配置
npm run query    # 查询配置
npm run pause    # 暂停费用收取
npm run resume   # 恢复费用收取
```

