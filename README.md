# Exchange rate API #

## Functioning and purpose ##

This API is used to get the exchange rate between USD and any asset in the Stellar network, based on swaps with USDC. This stablecoin is assumed to always be worth exactly $1. Exchange rates are calculated on 6-hour windows; the rates the API returns are represented on the basis of 1 USD – meaning, a value of 28 for an asset means $1 buys 28 units of that asset.

## Usage ##

### Shell ###

The API requires authentication via JWT. The general format for querying is as follows:

```bash
QUERY="{\"project_name\": \"kwickbit\", \"mode\": {\"Function\": {\"fname\": \"get_exchange_rate\", \"arguments\": \"$arguments\"}}}"

curl -s -X POST https://mainnet.mercurydata.app/zephyr/execute \
     -H "Authorization: Bearer $JWT" \
     -H 'Content-Type: application/json' \
     -d "$QUERY"
```

In the query, the variable `$arguments` has to be present. Due to parsing needs at the host server, it has to be double-escaped, like `"{\\\"asset_code\\\": \\\"XLM\\\", \\\"date\\\": \\\"2024-10-07T15:15:00\\\"}"`

### JavaScript ###

Here is sample JS code for querying the API. You must provide your Mercury JWT and the arguments explained below.
```js
const JWT = "your Mercury JWT here";  // Or read it from your .env file

const apiArgs = {
  asset_code: "EURC",
  // asset_issuer: "your_issuer_here",
  // date: "2024-10-07T15:15:00"
};

(async function queryAPI(args) {
  const response = await fetch("https://mainnet.mercurydata.app/zephyr/execute", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${JWT}`,
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

  console.log(await response.json());
})();
```

### Arguments ###

`asset_code`

**Mandatory**. Case-sensitive. Rates for this asset code will be returned.

`asset_issuer`

Optional. A 56-character string, starting with `G`, followed by numbers and uppercase letters. There is usually only one "canonical" issuer for a given asset, but there are valid exceptions (e.g. EURC). If this argument is provided, then only rates for the given combination of asset code and issuer will be searched. Otherwise, all available issuers are returned, in decreasing order of USDC trade volume in the 6-hour window.

`date`

Optional. Must meet the format `2019-10-14T10:45:00`; notice that a space between the day and hour is invalid, with `T` being required. The exchange rates returned are the most recent prior to the given date; if this argument is not provided, the current time is used.

### Response ###

In line with host server usage, the status code in the raw response received is always 200. The actual response from the API can follow one of the following formats:

```JSONc
// Success
{
  "data": [
    {
      "asset_code": "EURC",
      "asset_issuer": "GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2",
      "base_currency": "USD",
      "date_time": "2024-10-15T12:39:20.000000000Z",
      "exchange_rate": "0.9072931269624842",
      "volume": "32632.862163199996"
    },
    {
      "asset_code": "EURC",
      "asset_issuer": "GAQRF3UGHBT6JYQZ7YSUYCIYWAF4T2SAA5237Q5LIQYJOHHFAWDXZ7NM",
      "base_currency": "USD",
      "date_time": "2024-10-15T12:39:20.000000000Z",
      "exchange_rate": "1.0652092556689798",
      "volume": "11404.5021919"
    },
    {
      "asset_code": "EURC",
      "asset_issuer": "GAP2JFYUBSSY65FIFUN3NTUKP6MQQ52QETQEBDM25PFMQE2EEN2EEURC",
      "base_currency": "USD",
      "date_time": "2024-10-15T12:19:17.000000000Z",
      "exchange_rate": "0.9201194727031821",
      "volume": "0.0044529"
    }
  ],
  "status": 200
}

// Exchange rate not found
{
  "data": {
    "error": "No exchange rate found."
  },
  "status": 404
}

// Invalid date format provided
{
  "data": {
    "error": "Invalid date format. Please use the format '2020-09-16T14:30:00'."
  },
  "status": 400
}

// Internal server error
{
  "data": {
    "error": "An error occurred while querying the database."
  },
  "status": 500
}
```

Notice the root object in the response body always has exactly the two properties `status` and `data`, corresponding to the usual REST API structure.