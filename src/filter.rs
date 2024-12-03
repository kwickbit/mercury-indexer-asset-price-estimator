use zephyr_sdk::soroban_sdk::xdr::{
    ContractEvent, ContractEventBody, OperationResultTr, ScVal, ScVec, TransactionResultMeta,
    TransactionResultResult,
};
use zephyr_sdk::EnvClient;

use crate::config::SOROSWAP_ROUTER;
use crate::db::swap::{Swap, SwapData};
use crate::utils::{
    extract_claim_atom_data, extract_transaction_results, get_address_from_scval,
    get_claims_from_operation, get_swap_asset, hash_to_strkey, scmap_get,
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
 * only on Soroswap swaps; classic swaps are handled separately.
 */
pub(crate) fn soroswap_swaps(soroban_events: Vec<ContractEvent>) -> Vec<Swap> {
    soroban_events
        .into_iter()
        .filter_map(soroswap_event)
        .flat_map(swaps_from_event)
        .collect()
}

fn soroswap_event(event: ContractEvent) -> Option<ScVal> {
    let event_contract = event.contract_id.as_ref().map(hash_to_strkey)?;
    let ContractEventBody::V0(body) = event.body;

    let is_swap = body
        .topics
        .iter()
        .any(|topic| matches!(topic, ScVal::Symbol(s) if s.to_string() == "swap"));

    if event_contract == SOROSWAP_ROUTER && is_swap {
        EnvClient::empty().log().debug(
            format!(
                "SoroswapRouter swap happened: {:?}",
                body.data
            ),
            None,
        );
    }

    (event_contract == SOROSWAP_ROUTER && is_swap).then_some(body.data)
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

fn swaps_from_event(event: ScVal) -> Vec<Swap> {
    let ScVal::Map(Some(map)) = event else {
        return vec![];
    };

    let path = scmap_get(&map, "path".to_string());
    let amounts = scmap_get(&map, "amounts".to_string());

    if path.is_none() || amounts.is_none() {
        return vec![];
    }

    swaps_from_path_and_amounts(path.unwrap(), amounts.unwrap())
}

fn swaps_from_path_and_amounts(assets_in_path: &ScVec, amounts_in_path: &ScVec) -> Vec<Swap> {
    assets_in_path
        .0
        .windows(2)
        .zip(amounts_in_path.0.windows(2))
        .filter_map(swap_from_amounts_and_assets)
        .collect()
}

use zephyr_sdk::{prelude::*, DatabaseDerive};

#[derive(Clone, DatabaseDerive)]
#[with_name("soroswap")]
struct Soroswap {
    pub swap: String,
}

fn swap_from_amounts_and_assets((amounts, assets): (&[ScVal], &[ScVal])) -> Option<Swap> {
    let (ScVal::I128(n1), ScVal::I128(n2)) = (&amounts[0], &amounts[1]) else {
        return None;
    };

    let amount_sold = (((n1.hi as i128) << 64) + n1.lo as i128).try_into().ok()?;
    let amount_bought = (((n2.hi as i128) << 64) + n2.lo as i128).try_into().ok()?;
    let asset_sold = get_swap_asset(get_address_from_scval(&assets[0])?)?;
    let asset_bought = get_swap_asset(get_address_from_scval(&assets[1])?)?;

    let swap_data = SwapData {
        amount_bought,
        amount_sold,
        asset_bought: Some(*asset_bought),
        asset_sold: Some(*asset_sold),
    };

    let soroswap = Soroswap {
        swap: format!(
            "Swap data: {} {} for {} {}",
            amount_sold, asset_sold.code, amount_bought, asset_bought.code
        ),
    };

    soroswap.put(&EnvClient::new());

    Swap::try_from(&swap_data).ok()
}
