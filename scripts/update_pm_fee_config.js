/**
 * Update Prediction Market Fee Config
 * 
 * Updates fee rates and distribution shares in the existing PM Fee Config.
 * 
 * Usage: node update_pm_fee_config.js [options]
 * 
 * Options:
 *   --minting-fee <bps>       Minting fee in basis points (default: no change)
 *   --redemption-fee <bps>   Redemption fee in basis points
 *   --taker-fee <bps>        Taker trading fee in basis points
 *   --maker-fee <bps>        Maker trading fee in basis points
 *   --protocol-share <bps>   Protocol share in basis points
 *   --maker-share <bps>      Maker reward share in basis points
 *   --creator-share <bps>    Creator share in basis points
 */

const { 
  Connection, 
  Keypair, 
  PublicKey, 
  Transaction, 
  TransactionInstruction,
  sendAndConfirmTransaction,
} = require('@solana/web3.js');
const config = require('./config');
const fs = require('fs');

// Configuration
const FUND_PROGRAM_ID = config.FUND_PROGRAM_ID;
const PM_FEE_CONFIG_SEED = Buffer.from('prediction_market_fee_config');
const FUND_IX_UPDATE_PM_FEE_CONFIG = 46;

const RPC_URL = process.env.RPC_URL || 'https://testnet-rpc.1024chain.com/rpc/';

// Look for key in multiple locations
function findKeypairPath() {
  const paths = [
    process.env.KEYPAIR_PATH,
    '/Users/chuciqin/Desktop/project1024/1024codebase/1024-chain/keys/faucet.json',
    process.env.HOME + '/1024chain-testnet/keys/faucet.json',
  ].filter(Boolean);
  
  for (const path of paths) {
    try {
      fs.accessSync(path);
      return path;
    } catch {}
  }
  throw new Error('Could not find faucet.json keypair file');
}

const KEYPAIR_PATH = findKeypairPath();

// Parse command line arguments
function parseArgs() {
  const args = process.argv.slice(2);
  const config = {};
  
  for (let i = 0; i < args.length; i += 2) {
    const key = args[i];
    const value = parseInt(args[i + 1]);
    
    switch (key) {
      case '--minting-fee':
        config.mintingFeeBps = value;
        break;
      case '--redemption-fee':
        config.redemptionFeeBps = value;
        break;
      case '--taker-fee':
        config.takerFeeBps = value;
        break;
      case '--maker-fee':
        config.makerFeeBps = value;
        break;
      case '--protocol-share':
        config.protocolShareBps = value;
        break;
      case '--maker-share':
        config.makerShareBps = value;
        break;
      case '--creator-share':
        config.creatorShareBps = value;
        break;
      default:
        console.error(`Unknown option: ${key}`);
        process.exit(1);
    }
  }
  
  return config;
}

/**
 * Serialize UpdatePredictionMarketFeeConfigArgs
 * Uses Option<u16> for each field (None = no change)
 * 
 * Format for each field: 
 * - 0x00 = None (1 byte)
 * - 0x01 + u16 = Some(value) (3 bytes)
 */
function serializeUpdateArgs(config) {
  const parts = [];
  
  // Instruction index
  parts.push(Buffer.from([FUND_IX_UPDATE_PM_FEE_CONFIG]));
  
  // Each field as Option<u16>
  const fields = [
    'mintingFeeBps',
    'redemptionFeeBps',
    'takerFeeBps',
    'makerFeeBps',
    'protocolShareBps',
    'makerShareBps',
    'creatorShareBps',
  ];
  
  for (const field of fields) {
    if (config[field] !== undefined) {
      // Some(value)
      const buf = Buffer.alloc(3);
      buf.writeUInt8(1, 0); // Some discriminator
      buf.writeUInt16LE(config[field], 1);
      parts.push(buf);
    } else {
      // None
      parts.push(Buffer.from([0]));
    }
  }
  
  return Buffer.concat(parts);
}

async function main() {
  const config = parseArgs();
  
  if (Object.keys(config).length === 0) {
    console.log('Usage: node update_pm_fee_config.js [options]');
    console.log('');
    console.log('Options:');
    console.log('  --minting-fee <bps>      Minting fee (e.g., 10 = 0.1%)');
    console.log('  --redemption-fee <bps>   Redemption fee');
    console.log('  --taker-fee <bps>        Taker trading fee');
    console.log('  --maker-fee <bps>        Maker trading fee');
    console.log('  --protocol-share <bps>   Protocol share (e.g., 7000 = 70%)');
    console.log('  --maker-share <bps>      Maker reward share');
    console.log('  --creator-share <bps>    Creator share');
    console.log('');
    console.log('Example:');
    console.log('  node update_pm_fee_config.js --minting-fee 20 --taker-fee 15');
    return;
  }
  
  console.log('='.repeat(70));
  console.log('Update Prediction Market Fee Config');
  console.log('='.repeat(70));
  
  console.log('\nChanges to apply:');
  for (const [key, value] of Object.entries(config)) {
    console.log(`  ${key}: ${value} bps (${value / 100}%)`);
  }
  
  // Connect
  const connection = new Connection(RPC_URL, 'confirmed');
  console.log(`\nConnected to: ${RPC_URL}`);
  
  // Load keypair
  const keypairData = JSON.parse(fs.readFileSync(KEYPAIR_PATH, 'utf-8'));
  const admin = Keypair.fromSecretKey(new Uint8Array(keypairData));
  console.log(`Admin: ${admin.publicKey.toBase58()}`);
  
  // Derive PDA
  const [pmFeeConfigPda] = PublicKey.findProgramAddressSync(
    [PM_FEE_CONFIG_SEED],
    FUND_PROGRAM_ID
  );
  console.log(`PM Fee Config PDA: ${pmFeeConfigPda.toBase58()}`);
  
  // Check if exists
  const existingConfig = await connection.getAccountInfo(pmFeeConfigPda);
  if (!existingConfig) {
    console.error('❌ PM Fee Config not initialized. Run init_pm_fee_config.js first.');
    process.exit(1);
  }
  
  // Build instruction
  const data = serializeUpdateArgs(config);
  console.log(`\nInstruction data (${data.length} bytes): ${data.toString('hex')}`);
  
  const updateInstruction = new TransactionInstruction({
    programId: FUND_PROGRAM_ID,
    keys: [
      { pubkey: admin.publicKey, isSigner: true, isWritable: false },
      { pubkey: pmFeeConfigPda, isSigner: false, isWritable: true },
    ],
    data,
  });
  
  // Send transaction
  console.log('\nSending transaction...');
  const tx = new Transaction().add(updateInstruction);
  
  try {
    const signature = await sendAndConfirmTransaction(connection, tx, [admin], {
      commitment: 'confirmed',
    });
    
    console.log('\n✅ Update successful!');
    console.log(`Signature: ${signature}`);
    console.log(`Explorer: https://testnet-scan.1024chain.com/tx/${signature}`);
  } catch (error) {
    console.error('\n❌ Transaction failed:');
    if (error.logs) {
      error.logs.forEach(log => console.error('  ', log));
    }
    console.error(error.message || error);
    process.exit(1);
  }
  
  console.log('\nRun query_pm_fee_config.js to verify changes.');
  console.log('='.repeat(70));
}

main().catch(console.error);

