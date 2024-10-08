# Exchange rate API #

## Functioning and purpose ##

This API is used to get the exchange rate between any asset in the Stellar network and the USD. It is based on swaps between USDC and the asset in question. Exchange rates are calculated on 6-hour windows. Any rates the API returns are represented on the basis of 1 USD – meaning, a value of 28 for an asset means $1 buys 28 units of that asset. The value of 1 USDC is assumed to be exactly $1 at all times.

## Usage ##

The API requires authentication via JWT. The general format for querying is as follows:

```bash
QUERY="{\"project_name\": \"indexer\", \"mode\": {\"Function\": {\"fname\": \"get_exchange_rate\", \"arguments\": \"$arguments\"}}}"

curl -s -X POST https://mainnet.mercurydata.app/zephyr/execute \
     -H "Authorization: Bearer $JWT" \
     -H 'Content-Type: application/json' \
     -d "$QUERY"
```

In the query, the variable `$arguments` has to be present. Due to parsing needs at the host server, it has to be double-escaped, like `"{\\\"asset\\\": \\\"XLM\\\", \\\"date\\\": \\\"2024-10-07T15:15:00\\\"}"`

In the example above, replace `XLM` with the asset whose exchange rate is desired; this argument is mandatory. The `date` argument is optional; if it is not provided, the latest available rate for the asset is returned. When making use of this option, the format `2019-10-14T10:45:00` must be followed; a space between the day and hour is invalid, with `T` being required.

### Response ###

In line with host server usage, the status code in the raw response received is always 200. The actual response from the API can follow one of the following three formats:

```JSONc
// Success
{
  "data": {
    "asset": "yXLM",
    "base_currency": "USD",
    "date_time": "2024-10-08T14:53:24.000000000Z",
    "exchange_rate": "11.02165865188488"
  },
  "status": 200
}

// Exchange rate not found
{
  "data": {
    "asset": "ZWD",
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
```

Notice the root object in the response body always has exactly the two properties `status` and `data`, corresponding to the usual REST API structure.