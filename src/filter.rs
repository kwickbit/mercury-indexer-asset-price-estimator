use zephyr_sdk::soroban_sdk::xdr::{
    ContractEvent, ContractEventBody, OperationResultTr, ScVal, ScVec, TransactionResultMeta,
    TransactionResultResult,
};
use zephyr_sdk::EnvClient;

use crate::db::swap::{Swap, SwapData};
use crate::utils::{
    extract_claim_atom_data, extract_transaction_results, get_claims_from_operation, hash_to_strkey,
};

/**
 * We 'fish' every swap from each ledger close. This function focuses only on
 * classic swaps; Soroban swaps from Soroswap are handled separately. There is
 * some Vec flattening here because:
 * - there are many transactions in each close;
 * - there are many operations in each transaction;
 * - there can be many swaps in each operation.
 *
 * An operation can have no swaps if it is a create account, create contract, etc.
 * If its result is an Offer or PathPayment type, it can have multiple swaps.
 */
pub(crate) fn swaps(transaction_results: Vec<TransactionResultMeta>) -> Vec<Swap> {
    transaction_results
        .iter()
        .filter(is_transaction_successful)
        .flat_map(swaps_from_transaction)
        .collect()
}

/**
 * We 'fish' every Soroswap swap from each ledger close. This function focuses
 * only on Soroswap swaps; lassic swaps are handled separately.
 */
pub(crate) fn soroswap_swaps(soroban_events: Vec<ContractEvent>) -> Vec<Swap> {
    let client = EnvClient::empty();

    let address = "CAG5LRYQ5JVEUI5TEID72EYOVX44TTUJT5BQR2J6J77FH65PCCFAJDDH";

    let filtered_events: Vec<ScVal> = soroban_events
        .into_iter()
        .filter(|event| event.contract_id.is_some())
        .filter_map(|event| {
            let strkey = hash_to_strkey(event.contract_id.as_ref().unwrap());
            let ContractEventBody::V0(event_body) = event.body.clone();
            let is_swap =
                matches!(event_body.topics.first().unwrap(), ScVal::Symbol(s) if s.to_string() == "swap");

            if strkey == address && is_swap {
                Some(event_body.data)
            } else {
                None
            }
        })
        .collect();

    if !filtered_events.is_empty() {
        client
            .log()
            .debug(format!("First event: {:?}", filtered_events[0]), None);
    }

    filtered_events.iter().flat_map(swaps_from_event).collect()
}

fn is_transaction_successful(transaction: &&TransactionResultMeta) -> bool {
    matches!(
        transaction.result.result.result,
        TransactionResultResult::TxSuccess(_)
    )
}

fn swaps_from_transaction(transaction: &TransactionResultMeta) -> Vec<Swap> {
    let operations = extract_transaction_results(transaction);
    operations.iter().flat_map(swaps_from_operation).collect()
}

fn swaps_from_operation(operation: &OperationResultTr) -> Vec<Swap> {
    let claims = get_claims_from_operation(operation);

    claims
        .iter()
        .filter_map(|claim| Swap::try_from(&extract_claim_atom_data(claim)).ok())
        .collect()
}

fn swaps_from_event(event: &ScVal) -> Vec<Swap> {
    let client = EnvClient::empty();

    match event {
        ScVal::Map(Some(map)) => {
            let path = map
                .0
                .iter()
                .find(|entry| matches!(&entry.key, ScVal::Symbol(s) if s.to_string() == "path"))
                .and_then(|entry| match &entry.val {
                    ScVal::Vec(Some(v)) => Some(v),
                    _ => None,
                });

            let amounts = map
                .0
                .iter()
                .find(|entry| matches!(&entry.key, ScVal::Symbol(s) if s.to_string() == "amounts"))
                .and_then(|entry| match &entry.val {
                    ScVal::Vec(Some(v)) => Some(v),
                    _ => None,
                });

            match (path, amounts) {
                (Some(path), Some(amounts)) => {
                    // Continue with processing
                    swaps_from_path_and_amounts(path, amounts)
                }
                _ => {
                    client.log().debug("Path or amounts not found", None);
                    vec![]
                }
            }
        }
        _ => {
            client.log().debug("Event is not a map", None);
            vec![]
        }
    }
}

fn swaps_from_path_and_amounts(assets_in_path: &ScVec, amounts_in_path: &ScVec) -> Vec<Swap> {
    assets_in_path
        .0
        .windows(2)
        .zip(amounts_in_path.0.windows(2))
        .filter_map(|(assets, amounts)| {
            // These would be converted to Asset (like in extract_claim_atom_data)
            let amount_sold = match &amounts[0] {
                ScVal::I128(n) => ((n.hi as i128) << 64) + n.lo as i128,
                _ => return None,
            };
            let amount_bought = match &amounts[1] {
                ScVal::I128(n) => ((n.hi as i128) << 64) + n.lo as i128,
                _ => return None,
            };

            // Convert ScVal addresses to Assets
            let asset_sold = todo!(); // TODO: Convert ScVal address to Asset
            let asset_bought = todo!(); // TODO: Convert ScVal address to Asset

            let swap_data = SwapData {
                asset_sold,
                amount_sold: amount_sold.try_into().unwrap(),
                asset_bought,
                amount_bought: amount_bought.try_into().unwrap(),
            };

            Swap::try_from(&swap_data).ok()
        })
        .collect()
}
