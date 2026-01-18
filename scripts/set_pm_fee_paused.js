/**
 * Set Prediction Market Fee Paused/Unpaused
 * 
 * Pauses or resumes fee collection for the prediction market.
 * When paused, no fees will be collected (all operations are free).
 * 
 * Usage: 
 *   node set_pm_fee_paused.js --paused true   # Pause fee collection
 *   node set_pm_fee_paused.js --paused false  # Resume fee collection
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
const FUND_IX_SET_PM_FEE_PAUSED = 47;

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

async function main() {
  // Parse --paused argument
  const args = process.argv.slice(2);
  let paused = null;
  
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--paused' && args[i + 1]) {
      paused = args[i + 1].toLowerCase() === 'true';
    }
  }
  
  if (paused === null) {
    console.log('Usage: node set_pm_fee_paused.js --paused <true|false>');
    console.log('');
    console.log('Examples:');
    console.log('  node set_pm_fee_paused.js --paused true   # Pause fee collection');
    console.log('  node set_pm_fee_paused.js --paused false  # Resume fee collection');
    return;
  }
  
  console.log('='.repeat(70));
  console.log(`Set PM Fee ${paused ? 'PAUSED' : 'ACTIVE'}`);
  console.log('='.repeat(70));
  
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
    console.error('❌ PM Fee Config not initialized.');
    process.exit(1);
  }
  
  // Build instruction
  // SetPredictionMarketFeePausedArgs { is_paused: bool }
  const data = Buffer.alloc(2);
  data.writeUInt8(FUND_IX_SET_PM_FEE_PAUSED, 0);
  data.writeUInt8(paused ? 1 : 0, 1);
  
  console.log(`\nSetting is_paused = ${paused}`);
  
  const instruction = new TransactionInstruction({
    programId: FUND_PROGRAM_ID,
    keys: [
      { pubkey: admin.publicKey, isSigner: true, isWritable: false },
      { pubkey: pmFeeConfigPda, isSigner: false, isWritable: true },
    ],
    data,
  });
  
  // Send transaction
  console.log('Sending transaction...');
  const tx = new Transaction().add(instruction);
  
  try {
    const signature = await sendAndConfirmTransaction(connection, tx, [admin], {
      commitment: 'confirmed',
    });
    
    console.log(`\n✅ Successfully set PM Fee to ${paused ? 'PAUSED 🔴' : 'ACTIVE 🟢'}`);
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
  
  console.log('\n' + '='.repeat(70));
}

main().catch(console.error);

