# QuDAG Exchange CLI Reference

Complete command reference for the QuDAG Exchange CLI, covering all aspects of rUv token management, fee configuration, immutable deployment, and agent verification.

## Table of Contents

- [Core Commands](#core-commands)
- [Account Management](#account-management)
- [Token Operations](#token-operations)
- [Fee Model Management](#fee-model-management)
- [Immutable Deployment](#immutable-deployment)
- [Agent Verification](#agent-verification)
- [Network Status](#network-status)
- [Examples and Workflows](#examples-and-workflows)

## Core Commands

All exchange commands are accessible through the main QuDAG CLI:

```bash
qudag exchange <COMMAND> [OPTIONS]
```

### Available Commands

| Command | Description | Status |
|---------|-------------|--------|
| `create-account` | Create a new exchange account | ✅ Implemented |
| `balance` | Check account balance | ✅ Implemented |
| `transfer` | Transfer rUv tokens between accounts | ✅ Implemented |
| `mint` | Mint new rUv tokens | ✅ Implemented |
| `burn` | Burn rUv tokens | ✅ Implemented |
| `accounts` | List all accounts | ✅ Implemented |
| `supply` | Show total rUv supply | ✅ Implemented |
| `status` | Show exchange network status | ✅ Implemented |
| `deploy-immutable` | Deploy exchange in immutable mode | ✅ Implemented |
| `configure-fees` | Configure dynamic fee model parameters | ✅ Implemented |
| `fee-status` | Show current fee model status and examples | ✅ Implemented |
| `immutable-status` | Show immutable deployment status | ✅ Implemented |
| `verify-agent` | Verify agent for reduced fees | ✅ Implemented |
| `update-usage` | Update agent usage statistics | ✅ Implemented |
| `calculate-fee` | Calculate fee for a transaction | ✅ Implemented |

## Account Management

### Create Account

Create a new exchange account for rUv token operations.

```bash
qudag exchange create-account --name <NAME>
```

**Options:**
- `--name <NAME>`: Account name or identifier (required)

**Example:**
```bash
# Create accounts for trading
qudag exchange create-account --name alice
qudag exchange create-account --name bob
qudag exchange create-account --name production_agent
```

### Check Balance

Check the rUv token balance for a specific account.

```bash
qudag exchange balance --account <ACCOUNT>
```

**Options:**
- `--account <ACCOUNT>`: Account ID to check (required)

**Example:**
```bash
# Check account balances
qudag exchange balance --account alice
qudag exchange balance --account bob
```

### List Accounts

List all accounts with optional formatting.

```bash
qudag exchange accounts [--format <FORMAT>]
```

**Options:**
- `--format <FORMAT>`: Output format (text, json) [default: text]

**Example:**
```bash
# List accounts in different formats
qudag exchange accounts
qudag exchange accounts --format json
```

## Token Operations

### Transfer Tokens

Transfer rUv tokens between accounts with automatic fee calculation.

```bash
qudag exchange transfer --from <FROM> --to <TO> --amount <AMOUNT> [--memo <MEMO>]
```

**Options:**
- `--from <FROM>`: Source account (required)
- `--to <TO>`: Destination account (required)
- `--amount <AMOUNT>`: Amount to transfer in rUv (required)
- `--memo <MEMO>`: Optional transaction memo

**Example:**
```bash
# Basic transfer
qudag exchange transfer --from alice --to bob --amount 1000

# Transfer with memo
qudag exchange transfer \
  --from alice \
  --to bob \
  --amount 5000 \
  --memo "Resource payment for compute job #123"
```

### Mint Tokens

Mint new rUv tokens to a specific account.

```bash
qudag exchange mint --account <ACCOUNT> --amount <AMOUNT>
```

**Options:**
- `--account <ACCOUNT>`: Target account for minting (required)
- `--amount <AMOUNT>`: Amount to mint in rUv (required)

**Example:**
```bash
# Mint tokens for new participants
qudag exchange mint --account alice --amount 10000
qudag exchange mint --account bob --amount 5000
```

### Burn Tokens

Burn rUv tokens from a specific account.

```bash
qudag exchange burn --account <ACCOUNT> --amount <AMOUNT>
```

**Options:**
- `--account <ACCOUNT>`: Source account for burning (required)
- `--amount <AMOUNT>`: Amount to burn in rUv (required)

**Example:**
```bash
# Burn excess tokens
qudag exchange burn --account alice --amount 1000
```

### Supply Information

Show total rUv supply and distribution information.

```bash
qudag exchange supply
```

**Example Output:**
```
💰 rUv Token Supply
├── Total Supply: 1,000,000 rUv
├── Circulating: 750,000 rUv
├── Locked/Staked: 200,000 rUv
└── Available for Mint: 50,000 rUv
```

## Fee Model Management

### Configure Fee Parameters

Configure the dynamic tiered fee model parameters.

```bash
qudag exchange configure-fees [OPTIONS]
```

**Options:**
- `--f-min <RATE>`: Minimum fee rate (0.1% = 0.001)
- `--f-max <RATE>`: Maximum fee rate for unverified (1.0% = 0.01)
- `--f-min-verified <RATE>`: Minimum fee rate for verified (0.25% = 0.0025)
- `--f-max-verified <RATE>`: Maximum fee rate for verified (0.50% = 0.005)
- `--time-constant-days <DAYS>`: Time constant in days (default 90 days = 3 months)
- `--usage-threshold <AMOUNT>`: Usage threshold in rUv (default 10000)

**Examples:**
```bash
# Configure basic fee parameters
qudag exchange configure-fees \
  --f-min 0.001 \
  --f-max 0.01

# Full configuration for production
qudag exchange configure-fees \
  --f-min 0.002 \
  --f-max 0.012 \
  --f-min-verified 0.003 \
  --f-max-verified 0.006 \
  --time-constant-days 90 \
  --usage-threshold 15000

# Conservative fee structure
qudag exchange configure-fees \
  --f-min 0.001 \
  --f-max 0.005 \
  --f-min-verified 0.002 \
  --f-max-verified 0.003
```

### Fee Status and Examples

Show current fee model status with optional examples.

```bash
qudag exchange fee-status [--examples] [--format <FORMAT>]
```

**Options:**
- `--examples`: Show fee examples for different agent types
- `--format <FORMAT>`: Output format (text, json) [default: text]

**Example:**
```bash
# Basic fee status
qudag exchange fee-status

# Show detailed examples
qudag exchange fee-status --examples

# JSON output for integration
qudag exchange fee-status --format json
```

### Calculate Fee

Calculate the fee for a specific transaction.

```bash
qudag exchange calculate-fee --account <ACCOUNT> --amount <AMOUNT>
```

**Options:**
- `--account <ACCOUNT>`: Account ID for fee calculation (required)
- `--amount <AMOUNT>`: Transaction amount in rUv (required)

**Example:**
```bash
# Calculate fees for different scenarios
qudag exchange calculate-fee --account alice --amount 1000
qudag exchange calculate-fee --account verified_agent --amount 10000
qudag exchange calculate-fee --account high_volume_trader --amount 100000
```

## Immutable Deployment

### Deploy Immutable Mode

Deploy the exchange in immutable mode with quantum-resistant locking.

```bash
qudag exchange deploy-immutable [--key-path <PATH>] [--grace-period <HOURS>]
```

**Options:**
- `--key-path <PATH>`: Path to signing key for immutable deployment
- `--grace-period <HOURS>`: Grace period in hours before immutable mode takes effect [default: 24]

**Examples:**
```bash
# Deploy with default 24-hour grace period
qudag exchange deploy-immutable

# Deploy with 1-hour grace period for testing
qudag exchange deploy-immutable --grace-period 1

# Deploy with custom signing key
qudag exchange deploy-immutable \
  --key-path /secure/keys/immutable_deploy.key \
  --grace-period 48
```

### Immutable Status

Show the current immutable deployment status.

```bash
qudag exchange immutable-status [--format <FORMAT>]
```

**Options:**
- `--format <FORMAT>`: Output format (text, json) [default: text]

**Example:**
```bash
# Basic status
qudag exchange immutable-status

# JSON status for monitoring
qudag exchange immutable-status --format json
```

**Example Output:**
```json
{
  "enabled": true,
  "locked": true,
  "enforced": false,
  "in_grace_period": true,
  "locked_at": "2025-06-22T15:30:00Z",
  "grace_period_seconds": 3600,
  "config_hash": "blake3:a1b2c3d4e5f6..."
}
```

## Agent Verification

### Verify Agent

Verify an agent for reduced fees using cryptographic proof.

```bash
qudag exchange verify-agent --account <ACCOUNT> --proof-path <PATH>
```

**Options:**
- `--account <ACCOUNT>`: Account ID to verify (required)
- `--proof-path <PATH>`: Path to verification proof file (required)

**Proof File Format:**
```json
{
  "agent_id": "alice",
  "verification_type": "kyc_document",
  "proof_hash": "blake3:verification_data_hash",
  "timestamp": "2025-06-22T15:30:00Z",
  "signature": "ml_dsa_signature_bytes",
  "metadata": {
    "issuer": "QuDAG Verification Authority",
    "expires": "2026-06-22T15:30:00Z"
  }
}
```

**Example:**
```bash
# Verify agent with proof file
qudag exchange verify-agent \
  --account alice \
  --proof-path verification_proofs/alice_kyc.json

# Verify production agent
qudag exchange verify-agent \
  --account production_agent_001 \
  --proof-path proofs/enterprise_verification.json
```

### Update Usage Statistics

Update an agent's monthly usage statistics for fee calculation.

```bash
qudag exchange update-usage --account <ACCOUNT> --usage <USAGE>
```

**Options:**
- `--account <ACCOUNT>`: Account ID (required)
- `--usage <USAGE>`: Monthly usage in rUv (required)

**Example:**
```bash
# Update usage for fee calculation
qudag exchange update-usage --account alice --usage 15000
qudag exchange update-usage --account high_volume_agent --usage 75000

# Reset usage (new period)
qudag exchange update-usage --account bob --usage 0
```

## Network Status

### Exchange Status

Show comprehensive exchange network status.

```bash
qudag exchange status
```

**Example Output:**
```
🏦 QuDAG Exchange Status
├── Network: QuDAG Mainnet
├── Chain ID: 1
├── Total Accounts: 1,247
├── Active Agents: 892
├── Verified Agents: 234 (26.2%)
├── 24h Volume: 847,291 rUv
├── 24h Transactions: 15,443
├── Average Fee Rate: 0.342%
├── Network Health: ✅ Healthy
└── Last Block: #2,847,392 (2 seconds ago)

💱 Fee Model Status
├── Enabled: ✅ Yes
├── F_min: 0.1% (0.001)
├── F_max: 1.0% (0.010)
├── F_min_verified: 0.25% (0.0025)
├── F_max_verified: 0.50% (0.005)
├── Time constant: 90 days
└── Usage threshold: 10,000 rUv/month

🔒 Immutable Deployment
├── Enabled: ✅ Yes
├── Locked: ✅ Yes
├── Enforced: ✅ Yes
├── Locked at: 2025-06-20T10:30:00Z
├── Grace period: 24 hours (completed)
└── Config hash: blake3:a1b2c3d4e5f6...
```

## Examples and Workflows

### Complete Setup Workflow

```bash
# 1. Create accounts
qudag exchange create-account --name alice
qudag exchange create-account --name bob

# 2. Mint initial tokens
qudag exchange mint --account alice --amount 10000
qudag exchange mint --account bob --amount 5000

# 3. Configure fee model
qudag exchange configure-fees \
  --f-min 0.001 \
  --f-max 0.008 \
  --f-min-verified 0.002 \
  --f-max-verified 0.004

# 4. Verify agents
qudag exchange verify-agent \
  --account alice \
  --proof-path alice_verification.json

# 5. Update usage statistics
qudag exchange update-usage --account alice --usage 20000

# 6. Test transfers
qudag exchange transfer --from alice --to bob --amount 1000

# 7. Check results
qudag exchange balance --account alice
qudag exchange balance --account bob
qudag exchange fee-status --examples
```

### Production Deployment Workflow

```bash
# 1. Configure production fee parameters
qudag exchange configure-fees \
  --f-min 0.002 \
  --f-max 0.010 \
  --f-min-verified 0.003 \
  --f-max-verified 0.005 \
  --time-constant-days 90 \
  --usage-threshold 15000

# 2. Verify configuration
qudag exchange fee-status --examples

# 3. Deploy in immutable mode with 48-hour grace period
qudag exchange deploy-immutable --grace-period 48

# 4. Monitor deployment status
qudag exchange immutable-status

# 5. Wait for grace period to complete
# (Configuration becomes permanently locked)

# 6. Verify final status
qudag exchange status
```

### Agent Verification Workflow

```bash
# 1. Create verification proof file
cat > agent_verification.json << EOF
{
  "agent_id": "production_agent",
  "verification_type": "enterprise_kyc",
  "proof_hash": "blake3:enterprise_verification_hash",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "signature": "ml_dsa_signature_bytes",
  "metadata": {
    "issuer": "Enterprise Verification Authority",
    "level": "premium",
    "expires": "2026-06-22T15:30:00Z"
  }
}
EOF

# 2. Verify the agent
qudag exchange verify-agent \
  --account production_agent \
  --proof-path agent_verification.json

# 3. Update usage statistics for better rates
qudag exchange update-usage \
  --account production_agent \
  --usage 50000

# 4. Calculate fees to see benefits
qudag exchange calculate-fee \
  --account production_agent \
  --amount 10000

# 5. Compare with unverified agent
qudag exchange calculate-fee \
  --account unverified_agent \
  --amount 10000
```

### High-Volume Trading Setup

```bash
# 1. Create high-volume trader account
qudag exchange create-account --name hft_trader_001

# 2. Mint substantial tokens
qudag exchange mint --account hft_trader_001 --amount 1000000

# 3. Verify agent for reduced fees
qudag exchange verify-agent \
  --account hft_trader_001 \
  --proof-path hft_verification.json

# 4. Set high usage for optimal rates
qudag exchange update-usage \
  --account hft_trader_001 \
  --usage 100000

# 5. Test fee calculations at different volumes
qudag exchange calculate-fee --account hft_trader_001 --amount 1000
qudag exchange calculate-fee --account hft_trader_001 --amount 10000
qudag exchange calculate-fee --account hft_trader_001 --amount 100000

# 6. Monitor status
qudag exchange status
```

## Error Handling

### Common Error Scenarios

**Insufficient Balance:**
```bash
$ qudag exchange transfer --from alice --to bob --amount 999999
Error: Insufficient balance. Account 'alice' has 5000 rUv, requested 999999 rUv
```

**Invalid Account:**
```bash
$ qudag exchange balance --account nonexistent
Error: Account 'nonexistent' not found
```

**Immutable Configuration:**
```bash
$ qudag exchange configure-fees --f-min 0.002
Error: Cannot modify configuration: system is immutably locked
```

**Invalid Fee Parameters:**
```bash
$ qudag exchange configure-fees --f-min 1.5
Error: f_min must be between 0 and 1 (150% is invalid)
```

### Best Practices

1. **Always check balances before transfers**
2. **Verify fee calculations for large transactions**
3. **Use immutable deployment for production systems**
4. **Regularly update usage statistics for optimal fees**
5. **Keep verification proofs secure and backed up**
6. **Monitor exchange status for network health**
7. **Use JSON output for programmatic integration**

## Integration Examples

### Shell Script Integration

```bash
#!/bin/bash
# Simple trading bot example

ACCOUNT="trading_bot_001"
MIN_BALANCE=1000

# Check balance
BALANCE=$(qudag exchange balance --account $ACCOUNT --format json | jq -r '.balance')

if [ "$BALANCE" -lt "$MIN_BALANCE" ]; then
    echo "Low balance: $BALANCE rUv"
    # Mint more tokens or alert
    qudag exchange mint --account $ACCOUNT --amount 5000
fi

# Execute trade
qudag exchange transfer \
  --from $ACCOUNT \
  --to market_maker \
  --amount 100 \
  --memo "Automated trade $(date)"
```

### Python Integration

```python
import json
import subprocess

def get_exchange_status():
    """Get exchange status as Python dict."""
    result = subprocess.run([
        'qudag', 'exchange', 'status', '--format', 'json'
    ], capture_output=True, text=True)
    
    return json.loads(result.stdout)

def calculate_fee(account, amount):
    """Calculate fee for a transaction."""
    result = subprocess.run([
        'qudag', 'exchange', 'calculate-fee',
        '--account', account,
        '--amount', str(amount),
        '--format', 'json'
    ], capture_output=True, text=True)
    
    return json.loads(result.stdout)

# Example usage
status = get_exchange_status()
fee_info = calculate_fee('alice', 1000)
print(f"Network health: {status['network_health']}")
print(f"Fee for 1000 rUv: {fee_info['fee_amount']} rUv")
```

This comprehensive CLI reference provides everything needed to effectively use the QuDAG Exchange system for quantum-resistant resource trading.