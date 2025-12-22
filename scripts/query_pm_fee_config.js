/**
 * Query Prediction Market Fee Config
 * 
 * This script reads and displays the current PM Fee configuration.
 * 
 * Run: node query_pm_fee_config.js
 */

const { Connection, PublicKey } = require('@solana/web3.js');
const fs = require('fs');

// ============================================================================
// Configuration
// ============================================================================

const FUND_PROGRAM_ID = new PublicKey('FPhDzu7yCDC1BBvzGwpM6dHHNQBPpKEv6Y3Ptdc7o3fJ');
const PM_FEE_CONFIG_SEED = Buffer.from('prediction_market_fee_config');
const PM_FEE_VAULT_SEED = Buffer.from('prediction_market_fee_vault');

const RPC_URL = process.env.RPC_URL || 'https://testnet-rpc.1024chain.com/rpc/';

// Expected discriminator: "PM_FEE_C" = 0x504D5F4645455F43
const EXPECTED_DISCRIMINATOR = Buffer.from([0x43, 0x5F, 0x45, 0x45, 0x46, 0x5F, 0x4D, 0x50]); // little-endian

// ============================================================================
// Deserialization
// ============================================================================

/**
 * Deserialize PredictionMarketFeeConfig account data
 * 
 * Layout (from state.rs):
 * - [0-7]    u64 discriminator
 * - [8-39]   [u8; 32] prediction_market_fee_vault (Pubkey)
 * - [40]     u8 bump
 * - [41-42]  u16 prediction_market_minting_fee_bps
 * - [43-44]  u16 prediction_market_redemption_fee_bps
 * - [45-46]  u16 prediction_market_trading_fee_taker_bps
 * - [47-48]  u16 prediction_market_trading_fee_maker_bps
 * - [49-50]  u16 prediction_market_settlement_fee_bps
 * - [51-52]  u16 prediction_market_protocol_share_bps
 * - [53-54]  u16 prediction_market_maker_reward_share_bps
 * - [55-56]  u16 prediction_market_creator_share_bps
 * - [57-64]  i64 prediction_market_total_minting_fee_e6
 * - ... (more fields follow)
 */
function deserializePMFeeConfig(data) {
  if (data.length < 100) {
    throw new Error(`Data too short: ${data.length} bytes`);
  }
  
  let offset = 0;
  
  // Discriminator (8 bytes)
  const discriminator = data.slice(offset, offset + 8);
  offset += 8;
  
  // Fee vault pubkey (32 bytes)
  const feeVault = new PublicKey(data.slice(offset, offset + 32));
  offset += 32;
  
  // Bump (1 byte)
  const bump = data.readUInt8(offset);
  offset += 1;
  
  // Fee rates (u16 each)
  const mintingFeeBps = data.readUInt16LE(offset);
  offset += 2;
  
  const redemptionFeeBps = data.readUInt16LE(offset);
  offset += 2;
  
  const tradingFeeTakerBps = data.readUInt16LE(offset);
  offset += 2;
  
  const tradingFeeMakerBps = data.readUInt16LE(offset);
  offset += 2;
  
  const settlementFeeBps = data.readUInt16LE(offset);
  offset += 2;
  
  // Distribution shares (u16 each)
  const protocolShareBps = data.readUInt16LE(offset);
  offset += 2;
  
  const makerRewardShareBps = data.readUInt16LE(offset);
  offset += 2;
  
  const creatorShareBps = data.readUInt16LE(offset);
  offset += 2;
  
  // Statistics (i64 each)
  const totalMintingFeeE6 = data.readBigInt64LE(offset);
  offset += 8;
  
  const totalRedemptionFeeE6 = data.readBigInt64LE(offset);
  offset += 8;
  
  const totalTradingFeeE6 = data.readBigInt64LE(offset);
  offset += 8;
  
  const totalSettlementFeeE6 = data.readBigInt64LE(offset);
  offset += 8;
  
  const totalMakerRewardE6 = data.readBigInt64LE(offset);
  offset += 8;
  
  const totalCreatorRewardE6 = data.readBigInt64LE(offset);
  offset += 8;
  
  // Authorized caller (32 bytes)
  const authorizedCaller = new PublicKey(data.slice(offset, offset + 32));
  offset += 32;
  
  // Authority (32 bytes)
  const authority = new PublicKey(data.slice(offset, offset + 32));
  offset += 32;
  
  // isPaused (1 byte bool)
  const isPaused = data.readUInt8(offset) !== 0;
  offset += 1;
  
  return {
    discriminator,
    feeVault,
    bump,
    fees: {
      mintingFeeBps,
      redemptionFeeBps,
      tradingFeeTakerBps,
      tradingFeeMakerBps,
      settlementFeeBps,
    },
    distribution: {
      protocolShareBps,
      makerRewardShareBps,
      creatorShareBps,
    },
    statistics: {
      totalMintingFeeE6,
      totalRedemptionFeeE6,
      totalTradingFeeE6,
      totalSettlementFeeE6,
      totalMakerRewardE6,
      totalCreatorRewardE6,
    },
    authorizedCaller,
    authority,
    isPaused,
  };
}

// ============================================================================
// Main
// ============================================================================

async function main() {
  console.log('='.repeat(70));
  console.log('Query Prediction Market Fee Config');
  console.log('='.repeat(70));
  
  const connection = new Connection(RPC_URL, 'confirmed');
  console.log(`\nConnected to: ${RPC_URL}`);
  
  // Derive PDAs
  const [pmFeeConfigPda, configBump] = PublicKey.findProgramAddressSync(
    [PM_FEE_CONFIG_SEED],
    FUND_PROGRAM_ID
  );
  
  const [pmFeeVaultPda, vaultBump] = PublicKey.findProgramAddressSync(
    [PM_FEE_VAULT_SEED],
    FUND_PROGRAM_ID
  );
  
  console.log(`\nPM Fee Config PDA: ${pmFeeConfigPda.toBase58()}`);
  console.log(`PM Fee Vault PDA:  ${pmFeeVaultPda.toBase58()}`);
  
  // Fetch config account
  console.log('\n--- Fetching Config Account ---');
  const configAccount = await connection.getAccountInfo(pmFeeConfigPda);
  
  if (!configAccount) {
    console.log('❌ PM Fee Config not initialized');
    console.log('\nRun init_pm_fee_config.js to initialize.');
    return;
  }
  
  console.log(`✅ Account found`);
  console.log(`  Owner: ${configAccount.owner.toBase58()}`);
  console.log(`  Size: ${configAccount.data.length} bytes`);
  console.log(`  Lamports: ${configAccount.lamports}`);
  
  // Deserialize
  try {
    const config = deserializePMFeeConfig(configAccount.data);
    
    console.log('\n--- Fee Configuration ---');
    console.log(`Minting Fee:        ${config.fees.mintingFeeBps} bps (${config.fees.mintingFeeBps / 100}%)`);
    console.log(`Redemption Fee:     ${config.fees.redemptionFeeBps} bps (${config.fees.redemptionFeeBps / 100}%)`);
    console.log(`Taker Trading Fee:  ${config.fees.tradingFeeTakerBps} bps (${config.fees.tradingFeeTakerBps / 100}%)`);
    console.log(`Maker Trading Fee:  ${config.fees.tradingFeeMakerBps} bps (${config.fees.tradingFeeMakerBps / 100}%)`);
    console.log(`Settlement Fee:     ${config.fees.settlementFeeBps} bps (${config.fees.settlementFeeBps / 100}%)`);
    
    console.log('\n--- Fee Distribution ---');
    console.log(`Protocol Share:     ${config.distribution.protocolShareBps} bps (${config.distribution.protocolShareBps / 100}%)`);
    console.log(`Maker Reward Share: ${config.distribution.makerRewardShareBps} bps (${config.distribution.makerRewardShareBps / 100}%)`);
    console.log(`Creator Share:      ${config.distribution.creatorShareBps} bps (${config.distribution.creatorShareBps / 100}%)`);
    
    const totalShares = config.distribution.protocolShareBps + 
                        config.distribution.makerRewardShareBps + 
                        config.distribution.creatorShareBps;
    console.log(`Total:              ${totalShares} bps (${totalShares / 100}%)`);
    
    console.log('\n--- Statistics ---');
    console.log(`Total Minting Fee:     $${Number(config.statistics.totalMintingFeeE6) / 1e6}`);
    console.log(`Total Redemption Fee:  $${Number(config.statistics.totalRedemptionFeeE6) / 1e6}`);
    console.log(`Total Trading Fee:     $${Number(config.statistics.totalTradingFeeE6) / 1e6}`);
    console.log(`Total Settlement Fee:  $${Number(config.statistics.totalSettlementFeeE6) / 1e6}`);
    console.log(`Total Maker Reward:    $${Number(config.statistics.totalMakerRewardE6) / 1e6}`);
    console.log(`Total Creator Reward:  $${Number(config.statistics.totalCreatorRewardE6) / 1e6}`);
    
    const totalFees = Number(config.statistics.totalMintingFeeE6) +
                      Number(config.statistics.totalRedemptionFeeE6) +
                      Number(config.statistics.totalTradingFeeE6) +
                      Number(config.statistics.totalSettlementFeeE6);
    console.log(`─────────────────────────────────`);
    console.log(`Total Fees Collected:  $${totalFees / 1e6}`);
    
    console.log('\n--- Access Control ---');
    console.log(`Authorized Caller: ${config.authorizedCaller.toBase58()}`);
    console.log(`Authority:         ${config.authority.toBase58()}`);
    console.log(`Is Paused:         ${config.isPaused ? '🔴 YES' : '🟢 NO'}`);
    
    console.log('\n--- Accounts ---');
    console.log(`Fee Vault: ${config.feeVault.toBase58()}`);
    console.log(`Bump:      ${config.bump}`);
    
  } catch (error) {
    console.error('\n❌ Failed to deserialize config:', error.message);
    console.log('\nRaw data (first 128 bytes):');
    console.log(configAccount.data.slice(0, 128).toString('hex'));
  }
  
  // Check vault balance
  console.log('\n--- Fee Vault Balance ---');
  const vaultAccount = await connection.getAccountInfo(pmFeeVaultPda);
  if (vaultAccount) {
    console.log(`Vault exists: ${vaultAccount.owner.toBase58()}`);
    // If it's a token account, parse the balance
    if (vaultAccount.data.length >= 72) {
      const amount = vaultAccount.data.readBigUInt64LE(64);
      console.log(`USDC Balance: $${Number(amount) / 1e6}`);
    }
  } else {
    console.log('Vault not yet created');
  }
  
  console.log('\n' + '='.repeat(70));
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});

