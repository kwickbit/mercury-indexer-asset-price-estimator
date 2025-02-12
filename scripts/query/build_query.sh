#! /bin/bash

build_query() {
  local command="$1"
  local asset_code="$2"
  local asset_issuer="$3"
  local date="$4"
  local cat_text="$5"

  local fname
  local arguments

  if [ "$command" = "all" ]; then
    fname="get_all_exchange_rates"
    arguments="{}"
  elif [ "$command" = "asset" ]; then
    fname="get_exchange_rate"
    # The way the host server handles arguments requires us to double-escape
    # the arguments
    arguments="{\\\"asset_code\\\": \\\"$asset_code\\\""
    [ -n "$asset_issuer" ] && arguments="$arguments, \\\"asset_issuer\\\": \\\"$asset_issuer\\\""
    [ -n "$date" ] && arguments="$arguments, \\\"date\\\": \\\"$date\\\""
    arguments="$arguments}"
  elif [ "$command" = "cat" ]; then
    fname="cat"
    arguments="{\\\"text\\\": \\\"$cat_text\\\"}"
  elif [ "$command" = "currencies" ]; then
    fname="get_all_currencies"
    arguments="{}"
  elif [ "$command" = "duplicates" ]; then
    fname="get_duplicate_swaps"
    arguments="{}"
  elif [ "$command" = "history" ]; then
    fname="get_exchange_rate_history"
    arguments=$(cat "${BASE_DIR}/scripts/query/history_request.json" | tr -d '\n' | sed 's/"/\\"/g')
  elif [ "$command" = "savepoint" ]; then
    fname="savepoint"
    arguments="{}"
  elif [ "$command" = "soroswaps" ]; then
    fname="get_soroswap_swaps"
    arguments="{}"
  fi
  echo "{\"project_name\": \"kwickbit\", \"mode\": {\"Function\": {\"fname\": \"$fname\", \"arguments\": \"$arguments\"}}}"
}
