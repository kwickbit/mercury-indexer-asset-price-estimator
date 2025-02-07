# Exchange Rate API

## Overview

This API provides USD exchange rates for assets on the Stellar network, calculated from USDC swaps. Exchange rates are computed over 60-minute windows, with USDC assumed to be worth exactly $1. All rates are expressed in terms of 1 USD (e.g., a rate of 28 means $1 buys 28 units of the asset).

## Installation

After cloning this repository, put the following environment variables in your `.env` file:

```
  MERCURY_BACKEND_ENDPOINT
  MERCURY_GRAPHQL_ENDPOINT
  MERCURY_JWT
  MERCURY_API_KEY
```

## API Reference

### Endpoint

```
POST https://mainnet.mercurydata.app/zephyr/execute/66
```

### Content Type

```
Content-Type: application/json
```

### Request Types

#### 1. Single Exchange Rate

Function name: `get_exchange_rate`

Parameters:
- `asset_code` (required) - Case-sensitive asset code
- `asset_issuer` (optional) - 56-character string starting with 'G'
- `date` (optional) - ISO format timestamp (e.g., `2024-12-14T10:45:00`)

#### 2. Historical Exchange Rates

Function name: `get_exchange_rate_history`

Accepts multiple assets and dates in a batch request.

## Usage Examples

### Shell

#### Single Rate Query
```bash
QUERY="{
  \"project_name\": \"kwickbit\",
  \"mode\": {
    \"Function\": {
      \"fname\": \"get_exchange_rate\",
      \"arguments\": \"{
        \\\"asset_code\\\": \\\"EURC\\\",
        \\\"date\\\": \\\"2024-10-15T13:09:20\\\"
      }\"
    }
  }
}"

curl -s -X POST https://mainnet.mercurydata.app/zephyr/execute/66 \
     -H 'Content-Type: application/json' \
     -d "$QUERY"
```

#### Multiple Rates Query
```bash
QUERY="{
  \"project_name\": \"kwickbit\",
  \"mode\": {
    \"Function\": {
      \"fname\": \"get_exchange_rate_history\",
      \"arguments\": \"{
        \\\"assets\\\": [
          {
            \\\"asset\\\": {
              \\\"asset_code\\\": \\\"XLM\\\"
            },
            \\\"transaction_dates\\\": [
              \\\"2024-11-09T16:01:00Z\\\",
              \\\"2024-11-15T16:01:00Z\\\"
            ],
            \\\"unrealized_date\\\": \\\"2024-12-16T03:05:00Z\\\"
          }
        ]
      }\"
    }
  }
}"
```

### JavaScript

```javascript
const apiArgs = {
  asset_code: "EURC",
  // asset_issuer: "your_issuer_here",
  // date: "2024-10-07T15:15:00"
};

async function queryAPI() {
  const response = await fetch("https://mainnet.mercurydata.app/zephyr/execute/66", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      project_name: "kwickbit",
      mode: {
        Function: {
          fname: "get_exchange_rate",
          arguments: JSON.stringify(apiArgs),
        },
      },
    }),
  });

  return await response.json();
}
```

## Response Format

### Single Rate Response

```json
{
  "status": 200,
  "data": [
    {
      "asset_code": "EURC",
      "asset_issuer": "GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2",
      "base_currency": "USD",
      "date_time": "2024-10-15T12:39:20.000000000Z",
      "exchange_rate": "0.9072931269624842",
      "volume": "32632.862163199996"
    }
  ]
}
```

### Multiple Rates Response

```json
{
  "status": 200,
  "data": {
    "successful_assets": [
      {
        "asset": {
          "asset_code": "XLM",
          "asset_issuer": "Native"
        },
        "transaction_rates": [
          {
            "transaction_date": "2024-11-09T16:01:00.000000000Z",
            "exchange_rate_date": "2024-11-09T16:00:15.000000000Z",
            "exchange_rate": "9.953191422967757"
          }
        ],
        "unrealized_rate": {
          "transaction_date": "2024-12-16T03:05:00Z",
          "exchange_rate_date": "2024-12-16T03:00:00Z",
          "exchange_rate": "10.123456789"
        }
      }
    ],
    "failed_assets": []
  }
}
```

### Error Responses

All error responses follow this format:
```json
{
  "status": <error_code>,
  "data": {
    "error": "<error_message>"
  }
}
```

Common error codes:
- 400: Invalid date format
- 404: Exchange rate not found
- 500: Internal server error
