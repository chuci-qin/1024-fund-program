/**
 * Unified Configuration for 1024 Fund Program Scripts
 * 
 * All scripts should import from this file for consistent configuration.
 * Supports environment variable overrides for deployment flexibility.
 * 
 * Usage:
 *   const { FUND_PROGRAM_ID, PM_PROGRAM_ID, RPC_URL } = require('./config');
 * 
 * Environment Variables:
 *   - FUND_PROGRAM_ID: Override Fund Program ID
 *   - PM_PROGRAM_ID: Override Prediction Market Program ID
 *   - VAULT_PROGRAM_ID: Override Vault Program ID
 *   - SOLANA_RPC_URL or RPC_URL: Override RPC endpoint
 *   - USDC_MINT: Override USDC mint address
 *   - KEYPAIR_PATH: Override admin keypair path
 */

const { PublicKey } = require('@solana/web3.js');

// ============================================================================
// Network Configuration
// ============================================================================

const RPC_URL = process.env.SOLANA_RPC_URL || process.env.RPC_URL || 'https://testnet-rpc.1024chain.com/rpc/';

// ============================================================================
// Program IDs (支持环境变量覆盖)
// ============================================================================

const FUND_PROGRAM_ID = new PublicKey(
    process.env.FUND_PROGRAM_ID || 'FPhDzu7yCDC1BBvzGwpM6dHHNQBPpKEv6Y3Ptdc7o3fJ'
);

const PM_PROGRAM_ID = new PublicKey(
    process.env.PM_PROGRAM_ID || '9hsG1DksmgadjjJTEEX7CdevQKYVkQag3mEratPRZXjv'
);

const VAULT_PROGRAM_ID = new PublicKey(
    process.env.VAULT_PROGRAM_ID || 'vR3BifKCa2TGKP2uhToxZAMYAYydqpesvKGX54gzFny'
);

const USDC_MINT = new PublicKey(
    process.env.USDC_MINT || '7pCrfxhcAEyTFDhrhKRtRS2iMvEYx2dtNE7NzwuU7SA9'
);

const TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL');

// ============================================================================
// Admin Keypair Paths (检查多个位置)
// ============================================================================

const KEYPAIR_PATHS = [
    process.env.KEYPAIR_PATH,
    '/Users/chuciqin/Desktop/project1024/1024codebase/1024-chain/keys/faucet.json',
    process.env.HOME + '/1024chain-testnet/keys/faucet.json',
].filter(Boolean);

// ============================================================================
// PDA Seeds
// ============================================================================

const SEEDS = {
    PM_FEE_CONFIG: Buffer.from('prediction_market_fee_config'),
    PM_FEE_VAULT: Buffer.from('prediction_market_fee_vault'),
    SPOT_FEE_CONFIG: Buffer.from('spot_trading_fee_config'),
};

// ============================================================================
// Fund Program Instruction Indices
// ============================================================================

const FUND_INSTRUCTIONS = {
    INIT_PM_FEE_CONFIG: 40,
    UPDATE_PM_FEE_CONFIG: 41,
    SET_PM_FEE_PAUSED: 42,
};

// ============================================================================
// Default Fee Configuration (in basis points)
// ============================================================================

const DEFAULT_PM_FEE_CONFIG = {
    mintingFeeBps: 10,           // 0.1%
    redemptionFeeBps: 10,        // 0.1%
    tradingFeeTakerBps: 10,      // 0.1%
    tradingFeeMakerBps: 0,       // 0% (maker gets rebate, not charged)
    protocolShareBps: 7000,      // 70%
    makerRewardShareBps: 2000,   // 20%
    creatorShareBps: 1000,       // 10%
};

// ============================================================================
// Exports
// ============================================================================

module.exports = {
    // Network
    RPC_URL,
    
    // Program IDs
    FUND_PROGRAM_ID,
    PM_PROGRAM_ID,
    VAULT_PROGRAM_ID,
    USDC_MINT,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    
    // Paths
    KEYPAIR_PATHS,
    
    // PDA Seeds
    SEEDS,
    
    // Instructions
    FUND_INSTRUCTIONS,
    
    // Default Config
    DEFAULT_PM_FEE_CONFIG,
};


