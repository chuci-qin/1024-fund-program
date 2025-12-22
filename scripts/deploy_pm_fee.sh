#!/bin/bash
#
# Deploy Prediction Market Fee System
# 
# This script performs the complete deployment of the PM Fee system:
# 1. Initializes the PredictionMarketFeeConfig PDA
# 2. Verifies the configuration
# 3. Optionally runs a test transaction
#
# Usage: ./deploy_pm_fee.sh
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║        Prediction Market Fee System Deployment                  ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

# Check if node is available
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is required but not installed."
    exit 1
fi

# Check if npm packages are installed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "Step 1: Initialize PM Fee Config"
echo "═══════════════════════════════════════════════════════════════════"
node init_pm_fee_config.js

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "Step 2: Query and Verify Configuration"
echo "═══════════════════════════════════════════════════════════════════"
node query_pm_fee_config.js

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "Step 3: Deployment Complete!"
echo "═══════════════════════════════════════════════════════════════════"
echo ""
echo "✅ PM Fee System is now ready!"
echo ""
echo "Default Fee Rates:"
echo "  • Minting Fee:    0.1% (10 bps)"
echo "  • Redemption Fee: 0.1% (10 bps)"
echo "  • Taker Fee:      0.1% (10 bps)"
echo "  • Maker Fee:      0.0% (0 bps)"
echo ""
echo "Fee Distribution:"
echo "  • Protocol:  70%"
echo "  • Maker:     20%"
echo "  • Creator:   10%"
echo ""
echo "Management Commands:"
echo "  • Query config:  node query_pm_fee_config.js"
echo "  • Update fees:   node update_pm_fee_config.js --minting-fee 20"
echo "  • Pause fees:    node set_pm_fee_paused.js --paused true"
echo ""
echo "═══════════════════════════════════════════════════════════════════"

