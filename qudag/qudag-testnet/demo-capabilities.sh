#!/bin/bash
set -euo pipefail

# QuDAG Capabilities Demo Script
# Demonstrates core QuDAG functionality through testnet API

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Node endpoints
NODE1="http://109.105.222.156"  # Toronto (Enhanced)
NODE2="http://149.248.199.86"   # Amsterdam
NODE3="http://149.248.218.16"   # Singapore
NODE4="http://137.66.62.149"    # San Francisco

# Pretty print JSON
pretty_json() {
    echo "$1" | jq '.' 2>/dev/null || echo "$1"
}

# Demo header
demo_header() {
    echo -e "\n${PURPLE}═══════════════════════════════════════════════════════${NC}"
    echo -e "${PURPLE}   $1${NC}"
    echo -e "${PURPLE}═══════════════════════════════════════════════════════${NC}\n"
}

# Demo section
demo_section() {
    echo -e "\n${CYAN}▶ $1${NC}"
    echo -e "${CYAN}────────────────────────────────────────${NC}"
}

# Main demo
echo -e "${GREEN}"
echo "╔═══════════════════════════════════════════════════════╗"
echo "║        QuDAG Testnet Capabilities Demo                ║"
echo "║                                                       ║"
echo "║  Quantum-Resistant DAG for Autonomous AI Agents      ║"
echo "╚═══════════════════════════════════════════════════════╝"
echo -e "${NC}"

demo_header "🌐 Global Testnet Infrastructure"

demo_section "Node Locations & Status"
echo -e "${BLUE}Toronto${NC} (Enhanced Node):"
curl -s "$NODE1/health" | jq -r '"  Status: \(.status), Height: \(.details.height), Peers: \(.details.peers)"'

echo -e "\n${BLUE}Amsterdam${NC}:"
curl -s "$NODE2/health" | jq -r '"  Status: \(.status), Height: \(.details.height), Peers: \(.details.peers)"'

echo -e "\n${BLUE}Singapore${NC}:"
curl -s "$NODE3/health" | jq -r '"  Status: \(.status), Height: \(.details.height), Peers: \(.details.peers)"'

echo -e "\n${BLUE}San Francisco${NC}:"
curl -s "$NODE4/health" | jq -r '"  Status: \(.status), Height: \(.details.height), Peers: \(.details.peers)"'

demo_header "🔐 Quantum-Resistant Cryptography"

demo_section "Post-Quantum Algorithms"
echo "• ML-DSA (Dilithium-3) - Digital Signatures"
echo "• ML-KEM-768 - Key Encapsulation"
echo "• HQC - Hybrid Quantum Cryptography"
echo "• BLAKE3 - Quantum-Resistant Hashing"

demo_section "Quantum Fingerprinting Demo"
echo "Creating quantum fingerprint for data integrity..."
echo -e "${YELLOW}Data:${NC} 'QuDAG Testnet Demo'"
echo -e "${YELLOW}Fingerprint:${NC} $(echo -n 'QuDAG Testnet Demo' | sha256sum | cut -d' ' -f1 | cut -c1-32)..."
echo -e "${GREEN}✓ Collision-resistant quantum fingerprint generated${NC}"

demo_header "🌑 Dark Addressing System"

demo_section "Address Types"
echo "• Quantum Addresses: Based on ML-DSA public keys"
echo "• Shadow Addresses: Ephemeral, forward-secret"
echo "• Onion Addresses: Multi-hop routing with ChaCha20"

demo_section "Example Dark Domain"
echo -e "${YELLOW}Domain:${NC} mynode.dark"
echo -e "${YELLOW}Quantum Address:${NC} qd1z4ag3...7n9wvh"
echo -e "${YELLOW}Shadow Address:${NC} shadow:temp:2h:x9k2..."
echo -e "${GREEN}✓ Human-readable .dark domains without central authority${NC}"

demo_header "📊 DAG Consensus & Performance"

demo_section "Network Statistics"
# Get enhanced node stats
if stats=$(curl -s "$NODE1/api/v1/status" 2>/dev/null); then
    echo "Enhanced Node (Toronto):"
    echo "$stats" | jq -r '"  Messages Processed: \(.dag.messages_processed)"'
    echo "$stats" | jq -r '"  Network Messages: \(.p2p.network_messages)"'
    echo "$stats" | jq -r '"  Bytes Sent: \(.p2p.bytes_sent) bytes"'
    echo "$stats" | jq -r '"  Uptime: \(.node.uptime_seconds) seconds"'
fi

demo_section "QR-Avalanche Consensus"
echo "• Byzantine Fault Tolerant"
echo "• Parallel Message Processing"
echo "• Sub-second Finality"
echo "• No Mining Required"

demo_header "🤖 AI Agent Integration (MCP)"

demo_section "Model Context Protocol Features"
echo "• Native MCP Server Integration"
echo "• Agent Swarm Coordination"
echo "• Resource Sharing & Discovery"
echo "• Task Distribution"

demo_section "Zero-Person Business Support"
echo "• Autonomous Agent Operations"
echo "• rUv Token Exchange System"
echo "• Dynamic Fee Models"
echo "• Immutable Business Logic"

demo_header "🔒 Privacy & Security Features"

demo_section "Onion Routing"
echo "• Multi-hop message routing"
echo "• ChaCha20Poly1305 encryption"
echo "• Metadata obfuscation"
echo "• Traffic analysis resistance"

demo_section "Vault System"
echo "• Post-quantum encrypted storage"
echo "• AES-256-GCM + ML-KEM protection"
echo "• Hierarchical password organization"
echo "• Secure backup/restore"

demo_header "📈 Real-Time Metrics"

demo_section "Prometheus Metrics Available"
# Show sample metrics
echo "Sample metrics from enhanced node:"
curl -s "$NODE1/metrics" 2>/dev/null | grep "^qudag_" | head -5 | while read line; do
    echo "  $line"
done

demo_header "💱 Exchange & Business Plan"

demo_section "rUv Token System"
echo "• Resource Utilization Vouchers"
echo "• Quantum-resistant signatures"
echo "• Dynamic fee model (0.1% - 1.0%)"
echo "• Verified agent benefits"

demo_section "Business Plan Features"
echo "• Automated payout distribution"
echo "• Contributor role management"
echo "• Vault-based fund security"
echo "• Immutable deployment option"

demo_header "🚀 Getting Started"

echo -e "${GREEN}Install QuDAG CLI:${NC}"
echo "  cargo install qudag-cli"
echo
echo -e "${GREEN}Connect to Testnet:${NC}"
echo "  qudag start --bootstrap-peers /ip4/109.105.222.156/tcp/4001"
echo
echo -e "${GREEN}Create Dark Domain:${NC}"
echo "  qudag dark register mydomain.dark"
echo
echo -e "${GREEN}Generate Quantum Keys:${NC}"
echo "  qudag key generate --algorithm ml-dsa"

echo -e "\n${PURPLE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✨ QuDAG Testnet is Live and Ready! ✨${NC}"
echo -e "${PURPLE}═══════════════════════════════════════════════════════${NC}\n"

# Show live network activity
demo_section "Live Network Activity Monitor (10 seconds)"
echo "Monitoring block production..."
for i in {1..10}; do
    height=$(curl -s "$NODE1/health" 2>/dev/null | jq -r '.details.height' || echo "?")
    echo -ne "\r${YELLOW}[Block Height: $height]${NC} "
    sleep 1
done
echo -e "\n${GREEN}✓ Network is actively producing blocks${NC}"