/**
 * Initialize Prediction Market Fee Config in Fund Program
 * 
 * This script initializes the PredictionMarketFeeConfig PDA with default fee rates.
 * 
 * Run: node init_pm_fee_config.js
 * 
 * Prerequisites:
 * - Fund Program must be deployed
 * - Admin keypair (faucet.json) must have SOL for transaction fees
 */

const { 
  Connection, 
  Keypair, 
  PublicKey, 
  Transaction, 
  TransactionInstruction,
  SystemProgram,
  sendAndConfirmTransaction,
  SYSVAR_RENT_PUBKEY,
} = require('@solana/web3.js');
const { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress } = require('@solana/spl-token');
const fs = require('fs');

// ============================================================================
// Configuration
// ============================================================================

// Program IDs (1024Chain Testnet)
const FUND_PROGRAM_ID = new PublicKey('FPhDzu7yCDC1BBvzGwpM6dHHNQBPpKEv6Y3Ptdc7o3fJ');
const PM_PROGRAM_ID = new PublicKey('FVtPQkdYvSNdpTA6QXYRcTBhDGgnufw2Enqmo2tQKr58');
const USDC_MINT = new PublicKey('7pCrfxhcAEyTFDhrhKRtRS2iMvEYx2dtNE7NzwuU7SA9');

// PDA Seeds
const PM_FEE_CONFIG_SEED = Buffer.from('prediction_market_fee_config');
const PM_FEE_VAULT_SEED = Buffer.from('prediction_market_fee_vault');

// Fund Program Instruction Index
const FUND_IX_INIT_PM_FEE_CONFIG = 40;

// Default Fee Configuration (in basis points)
const DEFAULT_CONFIG = {
  mintingFeeBps: 10,           // 0.1%
  redemptionFeeBps: 10,        // 0.1%
  tradingFeeTakerBps: 10,      // 0.1%
  tradingFeeMakerBps: 0,       // 0% (maker gets rebate, not charged)
  protocolShareBps: 7000,      // 70%
  makerRewardShareBps: 2000,   // 20%
  creatorShareBps: 1000,       // 10%
};

// RPC Configuration
const RPC_URL = process.env.RPC_URL || 'https://testnet-rpc.1024chain.com/rpc/';
// Look for key in multiple locations
const KEYPAIR_PATHS = [
  process.env.KEYPAIR_PATH,
  '/Users/chuciqin/Desktop/project1024/1024codebase/1024-chain/keys/faucet.json',
  process.env.HOME + '/1024chain-testnet/keys/faucet.json',
].filter(Boolean);

function findKeypairPath() {
  for (const path of KEYPAIR_PATHS) {
    try {
      fs.accessSync(path);
      return path;
    } catch {}
  }
  throw new Error('Could not find faucet.json keypair file');
}

const KEYPAIR_PATH = findKeypairPath();

// ============================================================================
// Serialization
// ============================================================================

/**
 * Serialize InitializePredictionMarketFeeConfigArgs
 * 
 * Layout (15 bytes total):
 * - [0]     u8  instruction index (40)
 * - [1-2]   u16 minting_fee_bps
 * - [3-4]   u16 redemption_fee_bps
 * - [5-6]   u16 trading_fee_taker_bps
 * - [7-8]   u16 trading_fee_maker_bps
 * - [9-10]  u16 protocol_share_bps
 * - [11-12] u16 maker_reward_share_bps
 * - [13-14] u16 creator_share_bps
 */
function serializeInitPMFeeConfigArgs(config) {
  const buffer = Buffer.alloc(1 + 7 * 2); // 1 + 14 = 15 bytes
  let offset = 0;
  
  // Instruction index
  buffer.writeUInt8(FUND_IX_INIT_PM_FEE_CONFIG, offset);
  offset += 1;
  
  // Fee rates (u16 little-endian)
  buffer.writeUInt16LE(config.mintingFeeBps, offset);
  offset += 2;
  
  buffer.writeUInt16LE(config.redemptionFeeBps, offset);
  offset += 2;
  
  buffer.writeUInt16LE(config.tradingFeeTakerBps, offset);
  offset += 2;
  
  buffer.writeUInt16LE(config.tradingFeeMakerBps, offset);
  offset += 2;
  
  // Distribution shares (u16 little-endian)
  buffer.writeUInt16LE(config.protocolShareBps, offset);
  offset += 2;
  
  buffer.writeUInt16LE(config.makerRewardShareBps, offset);
  offset += 2;
  
  buffer.writeUInt16LE(config.creatorShareBps, offset);
  
  return buffer;
}

// ============================================================================
// Main
// ============================================================================

async function main() {
  console.log('='.repeat(70));
  console.log('Initialize Prediction Market Fee Config');
  console.log('='.repeat(70));
  
  // Connect to RPC (disable WebSocket to avoid 405 errors)
  console.log(`\nConnecting to: ${RPC_URL}`);
  const connection = new Connection(RPC_URL, {
    commitment: 'confirmed',
    wsEndpoint: undefined,  // Disable WebSocket
    confirmTransactionInitialTimeout: 60000,
  });
  
  // Load admin keypair
  console.log(`Loading keypair from: ${KEYPAIR_PATH}`);
  if (!fs.existsSync(KEYPAIR_PATH)) {
    console.error(`❌ Keypair file not found: ${KEYPAIR_PATH}`);
    console.error('Set KEYPAIR_PATH environment variable or ensure faucet.json exists');
    process.exit(1);
  }
  
  const keypairData = JSON.parse(fs.readFileSync(KEYPAIR_PATH, 'utf-8'));
  const admin = Keypair.fromSecretKey(new Uint8Array(keypairData));
  console.log(`Admin: ${admin.publicKey.toBase58()}`);
  
  // Derive PDAs
  console.log('\n--- Deriving PDAs ---');
  
  const [pmFeeConfigPda, configBump] = PublicKey.findProgramAddressSync(
    [PM_FEE_CONFIG_SEED],
    FUND_PROGRAM_ID
  );
  console.log(`PM Fee Config PDA: ${pmFeeConfigPda.toBase58()} (bump: ${configBump})`);
  
  const [pmFeeVaultPda, vaultBump] = PublicKey.findProgramAddressSync(
    [PM_FEE_VAULT_SEED],
    FUND_PROGRAM_ID
  );
  console.log(`PM Fee Vault PDA: ${pmFeeVaultPda.toBase58()} (bump: ${vaultBump})`);
  
  // Check if already initialized
  console.log('\n--- Checking existing state ---');
  const existingConfig = await connection.getAccountInfo(pmFeeConfigPda);
  if (existingConfig) {
    console.log('⚠️  PM Fee Config already exists!');
    console.log(`  Owner: ${existingConfig.owner.toBase58()}`);
    console.log(`  Size: ${existingConfig.data.length} bytes`);
    console.log('\nUse update_pm_fee_config.js to modify the configuration.');
    return;
  }
  console.log('✓ PM Fee Config does not exist, proceeding with initialization');
  
  // Check admin balance
  const balance = await connection.getBalance(admin.publicKey);
  console.log(`Admin balance: ${balance / 1e9} SOL`);
  if (balance < 0.01 * 1e9) {
    console.error('❌ Insufficient balance for transaction fees');
    process.exit(1);
  }
  
  // Prepare instruction data
  console.log('\n--- Configuration ---');
  console.log(`Minting Fee:        ${DEFAULT_CONFIG.mintingFeeBps} bps (${DEFAULT_CONFIG.mintingFeeBps / 100}%)`);
  console.log(`Redemption Fee:     ${DEFAULT_CONFIG.redemptionFeeBps} bps (${DEFAULT_CONFIG.redemptionFeeBps / 100}%)`);
  console.log(`Taker Trading Fee:  ${DEFAULT_CONFIG.tradingFeeTakerBps} bps (${DEFAULT_CONFIG.tradingFeeTakerBps / 100}%)`);
  console.log(`Maker Trading Fee:  ${DEFAULT_CONFIG.tradingFeeMakerBps} bps (${DEFAULT_CONFIG.tradingFeeMakerBps / 100}%)`);
  console.log(`Protocol Share:     ${DEFAULT_CONFIG.protocolShareBps} bps (${DEFAULT_CONFIG.protocolShareBps / 100}%)`);
  console.log(`Maker Reward Share: ${DEFAULT_CONFIG.makerRewardShareBps} bps (${DEFAULT_CONFIG.makerRewardShareBps / 100}%)`);
  console.log(`Creator Share:      ${DEFAULT_CONFIG.creatorShareBps} bps (${DEFAULT_CONFIG.creatorShareBps / 100}%)`);
  
  // Validate shares sum to 100%
  const totalShares = DEFAULT_CONFIG.protocolShareBps + 
                      DEFAULT_CONFIG.makerRewardShareBps + 
                      DEFAULT_CONFIG.creatorShareBps;
  if (totalShares !== 10000) {
    console.error(`❌ Share distribution must sum to 10000 bps, got ${totalShares}`);
    process.exit(1);
  }
  console.log(`✓ Share distribution validates (${totalShares} bps = 100%)`);
  
  const instructionData = serializeInitPMFeeConfigArgs(DEFAULT_CONFIG);
  console.log(`\nInstruction data (${instructionData.length} bytes): ${instructionData.toString('hex')}`);
  
  // Build instruction
  // Accounts (from processor.rs:2589-2598):
  // 0. `[signer]` Authority (admin)
  // 1. `[writable]` PredictionMarketFeeConfig PDA
  // 2. `[writable]` Prediction Market Fee Vault PDA (Token Account)
  // 3. `[]` USDC Mint
  // 4. `[]` Prediction Market Program (authorized caller)
  // 5. `[]` Token Program
  // 6. `[]` System Program
  // 7. `[]` Rent Sysvar
  const initInstruction = new TransactionInstruction({
    programId: FUND_PROGRAM_ID,
    keys: [
      { pubkey: admin.publicKey, isSigner: true, isWritable: true },
      { pubkey: pmFeeConfigPda, isSigner: false, isWritable: true },
      { pubkey: pmFeeVaultPda, isSigner: false, isWritable: true },
      { pubkey: USDC_MINT, isSigner: false, isWritable: false },
      { pubkey: PM_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    data: instructionData,
  });
  
  // Send transaction
  console.log('\n--- Sending Transaction ---');
  const tx = new Transaction().add(initInstruction);
  
  try {
    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.lastValidBlockHeight = lastValidBlockHeight;
    tx.feePayer = admin.publicKey;
    
    // Send and confirm with polling (no WebSocket)
    const rawTx = tx.serialize();
    const signature = await connection.sendRawTransaction(rawTx, {
      skipPreflight: false,
      preflightCommitment: 'confirmed',
    });
    
    console.log(`Transaction sent: ${signature}`);
    console.log('Waiting for confirmation...');
    
    // Poll for confirmation
    let confirmed = false;
    for (let i = 0; i < 60; i++) {
      await new Promise(r => setTimeout(r, 1000));
      const status = await connection.getSignatureStatus(signature);
      if (status && status.value) {
        if (status.value.confirmationStatus === 'confirmed' || 
            status.value.confirmationStatus === 'finalized') {
          confirmed = true;
          break;
        }
        if (status.value.err) {
          throw new Error(`Transaction failed: ${JSON.stringify(status.value.err)}`);
        }
      }
      if (i % 5 === 0) {
        console.log(`  Polling... (${i}s)`);
      }
    }
    
    if (!confirmed) {
      throw new Error('Transaction confirmation timeout');
    }
    
    console.log('\n✅ Initialization successful!');
    console.log(`Signature: ${signature}`);
    console.log(`Explorer: https://testnet-scan.1024chain.com/tx/${signature}`);
  } catch (error) {
    console.error('\n❌ Transaction failed:');
    if (error.logs) {
      console.error('Program logs:');
      error.logs.forEach(log => console.error('  ', log));
    }
    console.error(error.message || error);
    process.exit(1);
  }
  
  // Verify initialization
  console.log('\n--- Verifying ---');
  const finalConfig = await connection.getAccountInfo(pmFeeConfigPda);
  if (finalConfig) {
    console.log('✅ PM Fee Config account created');
    console.log(`  Owner: ${finalConfig.owner.toBase58()}`);
    console.log(`  Size: ${finalConfig.data.length} bytes`);
    console.log(`  Lamports: ${finalConfig.lamports}`);
  } else {
    console.error('❌ PM Fee Config account not found after transaction');
  }
  
  const finalVault = await connection.getAccountInfo(pmFeeVaultPda);
  if (finalVault) {
    console.log('✅ PM Fee Vault account created');
    console.log(`  Owner: ${finalVault.owner.toBase58()}`);
    console.log(`  Size: ${finalVault.data.length} bytes`);
  } else {
    console.error('❌ PM Fee Vault account not found after transaction');
  }
  
  // Print summary
  console.log('\n' + '='.repeat(70));
  console.log('Summary');
  console.log('='.repeat(70));
  console.log(`PM Fee Config PDA: ${pmFeeConfigPda.toBase58()}`);
  console.log(`PM Fee Vault PDA:  ${pmFeeVaultPda.toBase58()}`);
  console.log('\nAdd these to your configuration files and environment variables.');
  console.log('='.repeat(70));
}

main().catch(err => {
  console.error('Fatal error:', err);
  process.exit(1);
});

