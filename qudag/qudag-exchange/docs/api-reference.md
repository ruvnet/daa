# QuDAG Exchange API Reference

## Overview

The QuDAG Exchange API provides programmatic access to all exchange functionality through a RESTful HTTP interface and WebSocket connections for real-time updates.

### Base URL

```
Production: https://api.qudag.io/v1
Testnet: https://testnet-api.qudag.io/v1
Local: http://localhost:3000/api/v1
```

### Authentication

The API uses Bearer token authentication:

```http
Authorization: Bearer <your-api-token>
```

To obtain an API token:

```bash
qudag-exchange-cli api-token create --name "My App" --permissions "read,trade"
```

### Rate Limiting

- **Authenticated requests**: 600 requests per minute
- **Unauthenticated requests**: 60 requests per minute
- **WebSocket connections**: 10 concurrent connections

Rate limit headers:
```http
X-RateLimit-Limit: 600
X-RateLimit-Remaining: 599
X-RateLimit-Reset: 1703001600
```

### Common Response Format

All responses follow this structure:

```json
{
  "success": true,
  "data": {
    // Response data
  },
  "error": null,
  "timestamp": "2024-01-01T00:00:00Z",
  "request_id": "req_abc123"
}
```

Error responses:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "INSUFFICIENT_BALANCE",
    "message": "Account has insufficient rUv balance",
    "details": {
      "required": 100,
      "available": 50
    }
  },
  "timestamp": "2024-01-01T00:00:00Z",
  "request_id": "req_abc123"
}
```

## Account Endpoints

### Create Account

Create a new account with quantum-resistant keys.

```http
POST /accounts
Content-Type: application/json

{
  "name": "alice",
  "key_type": "ml-dsa",
  "metadata": {
    "email": "alice@example.com",
    "description": "Primary trading account"
  }
}
```

Response:
```json
{
  "success": true,
  "data": {
    "account_id": "acc_7d8f9a0b1c2d",
    "address": "qd1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0",
    "public_key": {
      "type": "ml-dsa",
      "value": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----"
    },
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### Get Account Info

Retrieve account details.

```http
GET /accounts/{account_id}
```

Response:
```json
{
  "success": true,
  "data": {
    "account_id": "acc_7d8f9a0b1c2d",
    "address": "qd1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0",
    "balance": "1000.50",
    "staked_balance": "500.00",
    "reputation_score": 95,
    "created_at": "2024-01-01T00:00:00Z",
    "metadata": {
      "email": "alice@example.com",
      "verified": true
    }
  }
}
```

### List Accounts

List all accounts (requires admin permissions).

```http
GET /accounts?limit=20&offset=0&sort=created_at:desc
```

Query parameters:
- `limit`: Number of results (default: 20, max: 100)
- `offset`: Pagination offset
- `sort`: Sort field and order
- `filter`: Filter expression

Response:
```json
{
  "success": true,
  "data": {
    "accounts": [
      {
        "account_id": "acc_7d8f9a0b1c2d",
        "address": "qd1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0",
        "balance": "1000.50",
        "created_at": "2024-01-01T00:00:00Z"
      }
    ],
    "pagination": {
      "total": 150,
      "limit": 20,
      "offset": 0
    }
  }
}
```

### Update Account

Update account metadata.

```http
PATCH /accounts/{account_id}
Content-Type: application/json

{
  "metadata": {
    "email": "newemail@example.com",
    "notification_preferences": {
      "trades": true,
      "offers": false
    }
  }
}
```

## Balance Endpoints

### Get Balance

Get account balance.

```http
GET /accounts/{account_id}/balance
```

Response:
```json
{
  "success": true,
  "data": {
    "available": "1000.50",
    "staked": "500.00",
    "pending": "25.00",
    "total": "1525.50",
    "currency": "rUv",
    "last_updated": "2024-01-01T00:00:00Z"
  }
}
```

### Get Balance History

Retrieve historical balance data.

```http
GET /accounts/{account_id}/balance/history?period=7d&interval=1h
```

Query parameters:
- `period`: Time period (1h, 24h, 7d, 30d, 1y)
- `interval`: Data point interval
- `from`: Start timestamp
- `to`: End timestamp

Response:
```json
{
  "success": true,
  "data": {
    "history": [
      {
        "timestamp": "2024-01-01T00:00:00Z",
        "balance": "1000.00",
        "change": "+50.00"
      }
    ],
    "summary": {
      "start_balance": "950.00",
      "end_balance": "1000.50",
      "total_change": "+50.50",
      "percent_change": "+5.32"
    }
  }
}
```

## Transaction Endpoints

### Create Transaction

Submit a new transaction.

```http
POST /transactions
Content-Type: application/json

{
  "from": "acc_7d8f9a0b1c2d",
  "to": "qd1x2y3z4a5b6c7d8e9f0g1h2i3j4k5l6m7n8o9p0",
  "amount": "100.00",
  "memo": "Payment for compute resources",
  "fee": "0.01",
  "signature": "base64_encoded_signature"
}
```

Response:
```json
{
  "success": true,
  "data": {
    "transaction_id": "tx_9f8e7d6c5b4a",
    "status": "pending",
    "hash": "0x1234567890abcdef...",
    "created_at": "2024-01-01T00:00:00Z",
    "estimated_confirmation": "2024-01-01T00:00:05Z"
  }
}
```

### Get Transaction

Retrieve transaction details.

```http
GET /transactions/{transaction_id}
```

Response:
```json
{
  "success": true,
  "data": {
    "transaction_id": "tx_9f8e7d6c5b4a",
    "hash": "0x1234567890abcdef...",
    "from": "qd1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0",
    "to": "qd1x2y3z4a5b6c7d8e9f0g1h2i3j4k5l6m7n8o9p0",
    "amount": "100.00",
    "fee": "0.01",
    "memo": "Payment for compute resources",
    "status": "confirmed",
    "confirmations": 6,
    "block_height": 123456,
    "created_at": "2024-01-01T00:00:00Z",
    "confirmed_at": "2024-01-01T00:00:05Z"
  }
}
```

### List Transactions

List transactions with filters.

```http
GET /transactions?account=acc_7d8f9a0b1c2d&type=sent&status=confirmed&limit=50
```

Query parameters:
- `account`: Filter by account
- `type`: Transaction type (sent, received, all)
- `status`: Status filter (pending, confirmed, failed)
- `from_date`: Start date
- `to_date`: End date
- `min_amount`: Minimum amount
- `max_amount`: Maximum amount

### Batch Transactions

Submit multiple transactions atomically.

```http
POST /transactions/batch
Content-Type: application/json

{
  "transactions": [
    {
      "from": "acc_7d8f9a0b1c2d",
      "to": "qd1recipient1...",
      "amount": "50.00"
    },
    {
      "from": "acc_7d8f9a0b1c2d",
      "to": "qd1recipient2...",
      "amount": "30.00"
    }
  ],
  "atomic": true,
  "signature": "base64_encoded_signature"
}
```

## Resource Trading Endpoints

### Create Resource Offer

List resources for trading.

```http
POST /resources/offers
Content-Type: application/json

{
  "provider": "acc_7d8f9a0b1c2d",
  "resource_type": "compute",
  "specifications": {
    "cpu": 8,
    "memory": "32GB",
    "gpu": "RTX 4090",
    "gpu_count": 2
  },
  "pricing": {
    "price_per_hour": "50.00",
    "currency": "rUv",
    "minimum_duration": "1h",
    "maximum_duration": "24h"
  },
  "availability": {
    "start": "2024-01-01T00:00:00Z",
    "end": "2024-01-31T23:59:59Z",
    "schedule": "24/7"
  },
  "sla": {
    "uptime_guarantee": 0.999,
    "support_response_time": "1h"
  }
}
```

Response:
```json
{
  "success": true,
  "data": {
    "offer_id": "offer_3c2b1a0f9e8d",
    "status": "active",
    "visibility": "public",
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### Search Resource Offers

Find available resources.

```http
GET /resources/offers/search
```

Query parameters:
```http
GET /resources/offers/search?
  type=compute&
  min_cpu=4&
  min_memory=16GB&
  gpu_type=RTX%204090&
  max_price=100&
  location=US-EAST&
  sort=price:asc
```

Response:
```json
{
  "success": true,
  "data": {
    "offers": [
      {
        "offer_id": "offer_3c2b1a0f9e8d",
        "provider": {
          "account_id": "acc_7d8f9a0b1c2d",
          "reputation": 95,
          "completed_jobs": 1523
        },
        "resource_type": "compute",
        "specifications": {
          "cpu": 8,
          "memory": "32GB",
          "gpu": "RTX 4090"
        },
        "pricing": {
          "price_per_hour": "50.00",
          "currency": "rUv"
        },
        "availability": {
          "immediate": true,
          "slots": ["2024-01-01T10:00:00Z", "2024-01-01T14:00:00Z"]
        },
        "match_score": 0.95
      }
    ],
    "pagination": {
      "total": 42,
      "limit": 20,
      "offset": 0
    }
  }
}
```

### Create Resource Reservation

Reserve resources from an offer.

```http
POST /resources/reservations
Content-Type: application/json

{
  "offer_id": "offer_3c2b1a0f9e8d",
  "consumer": "acc_1a2b3c4d5e6f",
  "duration": "4h",
  "start_time": "2024-01-01T10:00:00Z",
  "auto_renew": true,
  "payment": {
    "method": "escrow",
    "total_amount": "200.00"
  }
}
```

Response:
```json
{
  "success": true,
  "data": {
    "reservation_id": "res_5e4d3c2b1a0f",
    "status": "confirmed",
    "access_credentials": {
      "endpoint": "compute.provider123.qudag.io",
      "auth_token": "encrypted_token",
      "ssh_key": "encrypted_ssh_key"
    },
    "escrow": {
      "transaction_id": "tx_escrow_9f8e7d",
      "amount": "200.00",
      "release_conditions": "automatic_on_completion"
    }
  }
}
```

### Get Resource Usage

Monitor resource consumption.

```http
GET /resources/reservations/{reservation_id}/usage
```

Response:
```json
{
  "success": true,
  "data": {
    "reservation_id": "res_5e4d3c2b1a0f",
    "current_usage": {
      "cpu_hours": 3.5,
      "memory_gb_hours": 112,
      "gpu_hours": 7,
      "bandwidth_gb": 45.3
    },
    "cost_breakdown": {
      "cpu_cost": "3.50",
      "memory_cost": "11.20",
      "gpu_cost": "350.00",
      "bandwidth_cost": "0.45",
      "total_cost": "365.15"
    },
    "remaining_credit": "34.85",
    "usage_percentage": 91.29,
    "last_updated": "2024-01-01T13:30:00Z"
  }
}
```

## Provider Endpoints

### Register as Provider

Register resource provider capabilities.

```http
POST /providers/register
Content-Type: application/json

{
  "account_id": "acc_7d8f9a0b1c2d",
  "provider_info": {
    "name": "QuantumCompute Pro",
    "description": "High-performance GPU cluster",
    "location": {
      "region": "US-EAST",
      "city": "New York",
      "coordinates": {
        "lat": 40.7128,
        "lon": -74.0060
      }
    }
  },
  "capabilities": {
    "compute": {
      "cpu_cores": 256,
      "memory_gb": 1024,
      "gpus": [
        {
          "model": "RTX 4090",
          "count": 8,
          "memory_gb": 24
        },
        {
          "model": "A100",
          "count": 4,
          "memory_gb": 80
        }
      ]
    },
    "storage": {
      "total_tb": 100,
      "type": "NVMe SSD",
      "iops": 1000000
    },
    "network": {
      "bandwidth_gbps": 10,
      "latency_ms": 5
    }
  },
  "certifications": ["ISO27001", "SOC2"],
  "stake_amount": "10000.00"
}
```

### Update Provider Status

Update provider availability and pricing.

```http
PATCH /providers/{provider_id}/status
Content-Type: application/json

{
  "status": "maintenance",
  "maintenance_window": {
    "start": "2024-01-01T02:00:00Z",
    "end": "2024-01-01T04:00:00Z",
    "reason": "System upgrade"
  },
  "affected_resources": ["gpu"],
  "notification_sent": true
}
```

### Get Provider Stats

Retrieve provider performance statistics.

```http
GET /providers/{provider_id}/stats?period=30d
```

Response:
```json
{
  "success": true,
  "data": {
    "provider_id": "prov_9e8d7c6b5a4f",
    "period": "30d",
    "performance": {
      "uptime_percentage": 99.95,
      "average_response_time_ms": 250,
      "completed_jobs": 1523,
      "failed_jobs": 2,
      "customer_satisfaction": 4.8
    },
    "earnings": {
      "total_earned": "45678.90",
      "daily_average": "1522.63",
      "pending_payments": "1234.56"
    },
    "resource_utilization": {
      "cpu_utilization": 0.75,
      "memory_utilization": 0.82,
      "gpu_utilization": 0.91,
      "storage_utilization": 0.45
    },
    "top_clients": [
      {
        "client_id": "acc_1a2b3c4d5e6f",
        "total_spent": "5678.90",
        "job_count": 234
      }
    ]
  }
}
```

## Market Data Endpoints

### Get Market Overview

Retrieve market statistics.

```http
GET /market/overview
```

Response:
```json
{
  "success": true,
  "data": {
    "total_providers": 1543,
    "active_offers": 8921,
    "24h_volume": "1234567.89",
    "average_prices": {
      "cpu_per_hour": "1.23",
      "gpu_per_hour": "45.67",
      "storage_per_gb_month": "0.05",
      "bandwidth_per_gb": "0.001"
    },
    "supply_demand": {
      "cpu": {
        "supply": 125000,
        "demand": 98000,
        "ratio": 1.28
      },
      "gpu": {
        "supply": 2500,
        "demand": 3100,
        "ratio": 0.81
      }
    },
    "trending_resources": [
      {
        "type": "gpu",
        "model": "H100",
        "demand_increase": "+45%"
      }
    ]
  }
}
```

### Get Price History

Retrieve historical pricing data.

```http
GET /market/prices/history?resource=gpu&model=RTX%204090&period=7d&interval=1h
```

Response:
```json
{
  "success": true,
  "data": {
    "resource": "gpu",
    "model": "RTX 4090",
    "period": "7d",
    "interval": "1h",
    "price_points": [
      {
        "timestamp": "2024-01-01T00:00:00Z",
        "price": "45.00",
        "volume": "1234",
        "trades": 56
      }
    ],
    "statistics": {
      "average": "46.78",
      "median": "45.50",
      "high": "52.00",
      "low": "42.00",
      "volatility": 0.082
    }
  }
}
```

## Network Endpoints

### Get Network Status

Retrieve network health and statistics.

```http
GET /network/status
```

Response:
```json
{
  "success": true,
  "data": {
    "network_health": "healthy",
    "node_count": 3456,
    "active_validators": 100,
    "consensus": {
      "algorithm": "QR-Avalanche",
      "finality_time_seconds": 2.5,
      "tps": 8543,
      "pending_transactions": 234
    },
    "p2p": {
      "connected_peers": 45,
      "bandwidth_usage_mbps": 234.5,
      "message_propagation_ms": 125
    },
    "chain": {
      "height": 1234567,
      "hash": "0xabcdef...",
      "timestamp": "2024-01-01T00:00:00Z"
    }
  }
}
```

### Get Peer List

List connected network peers.

```http
GET /network/peers?limit=10&sort=latency:asc
```

Response:
```json
{
  "success": true,
  "data": {
    "peers": [
      {
        "peer_id": "QmPeer123...",
        "address": "/ip4/1.2.3.4/tcp/8080",
        "latency_ms": 15,
        "uptime_hours": 720,
        "reputation": 98,
        "capabilities": ["validator", "storage", "compute"]
      }
    ],
    "total_peers": 45,
    "average_latency_ms": 32
  }
}
```

## WebSocket API

### Connection

Connect to WebSocket endpoint:

```javascript
const ws = new WebSocket('wss://api.qudag.io/v1/ws');

ws.onopen = () => {
  // Authenticate
  ws.send(JSON.stringify({
    type: 'auth',
    token: 'your_api_token'
  }));
};
```

### Subscriptions

Subscribe to real-time updates:

```javascript
// Subscribe to account updates
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'account',
  params: {
    account_id: 'acc_7d8f9a0b1c2d'
  }
}));

// Subscribe to market data
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'market',
  params: {
    resource_types: ['gpu', 'cpu'],
    price_updates: true
  }
}));

// Subscribe to transactions
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'transactions',
  params: {
    account_id: 'acc_7d8f9a0b1c2d',
    include_pending: true
  }
}));
```

### Message Types

Account balance update:
```json
{
  "type": "account.balance",
  "data": {
    "account_id": "acc_7d8f9a0b1c2d",
    "balance": "1050.25",
    "change": "+50.25",
    "transaction_id": "tx_9f8e7d6c5b4a"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

Transaction confirmation:
```json
{
  "type": "transaction.confirmed",
  "data": {
    "transaction_id": "tx_9f8e7d6c5b4a",
    "confirmations": 6,
    "final": true
  },
  "timestamp": "2024-01-01T00:00:05Z"
}
```

Market price update:
```json
{
  "type": "market.price",
  "data": {
    "resource": "gpu",
    "model": "RTX 4090",
    "price": "46.50",
    "change_24h": "+2.3%",
    "volume_24h": "23456"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Error Codes

| Code | Description | HTTP Status |
|------|-------------|-------------|
| `UNAUTHORIZED` | Missing or invalid authentication | 401 |
| `FORBIDDEN` | Insufficient permissions | 403 |
| `NOT_FOUND` | Resource not found | 404 |
| `VALIDATION_ERROR` | Invalid request parameters | 400 |
| `INSUFFICIENT_BALANCE` | Not enough rUv tokens | 400 |
| `RATE_LIMITED` | Too many requests | 429 |
| `INTERNAL_ERROR` | Server error | 500 |
| `SERVICE_UNAVAILABLE` | Service temporarily down | 503 |

## SDK Examples

### JavaScript/TypeScript

```typescript
import { QuDagExchangeSDK } from '@qudag/exchange-sdk';

const sdk = new QuDagExchangeSDK({
  apiKey: 'your_api_key',
  network: 'mainnet'
});

// Create transaction
const tx = await sdk.transactions.create({
  to: 'qd1recipient...',
  amount: '100.00',
  memo: 'Payment'
});

// Search for GPU resources
const offers = await sdk.resources.search({
  type: 'compute',
  gpuModel: 'RTX 4090',
  maxPrice: 50
});

// Subscribe to updates
sdk.websocket.subscribe('account.balance', (data) => {
  console.log('Balance updated:', data.balance);
});
```

### Python

```python
from qudag_exchange import Client

client = Client(api_key='your_api_key')

# Get account balance
balance = client.accounts.get_balance('acc_7d8f9a0b1c2d')

# Create resource offer
offer = client.resources.create_offer(
    resource_type='compute',
    specifications={
        'cpu': 8,
        'memory': '32GB',
        'gpu': 'RTX 4090'
    },
    price_per_hour=50.0
)

# Monitor transactions
for tx in client.transactions.stream(account='acc_7d8f9a0b1c2d'):
    print(f"New transaction: {tx.id} - {tx.amount} rUv")
```

### Rust

```rust
use qudag_exchange_sdk::{Client, ResourceQuery};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("your_api_key")?;
    
    // Transfer tokens
    let tx = client.transfer()
        .to("qd1recipient...")
        .amount("100.00")
        .memo("Payment")
        .send()
        .await?;
    
    // Find resources
    let offers = client.resources()
        .search(ResourceQuery {
            resource_type: ResourceType::Compute,
            min_gpu_count: Some(2),
            max_price: Some(50.0),
            ..Default::default()
        })
        .await?;
    
    Ok(())
}
```

## API Versioning

The API uses URL versioning. Current version: `v1`

When breaking changes are introduced:
1. New version endpoint is created (e.g., `/v2`)
2. Previous version supported for 6 months
3. Deprecation warnings added to responses
4. Migration guide published

Version compatibility:
```http
X-API-Version: 1
X-API-Deprecated: false
X-API-Sunset-Date: 2025-01-01
```