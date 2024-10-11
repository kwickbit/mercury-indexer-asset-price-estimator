use std::vec;

use zephyr_sdk::soroban_sdk::xdr::{
    ManageBuyOfferResult, ManageOfferSuccessResult, ManageOfferSuccessResultOffer, ManageSellOfferResult, OperationResultTr, PathPaymentStrictReceiveResult, PathPaymentStrictSendResult, TransactionResultMeta, TransactionResultResult
};
use zephyr_sdk::EnvClient;

use crate::config::USDC;
use crate::db::swap::Swap;
use crate::utils::{
    extract_claim_atoms_from_path_payment_result, extract_transaction_results,
};

/*
We fish every swap from each ledger close. There is a lot of Vec flattening because:
- there are many transactions in each close;
- there are many operations in each transaction;
- if the operation is a path payment, it can have more than one swap (as in,
  an asset is traded for USDC and that is traded for something else).

An operation can have no swaps if it is a create account, create contract, etc.
If its result is of an Offer type, it can have one swap, if one of the assets
involved is USDC.
Path payments can have more than one swap so we always build Vecs of swaps.
*/
pub fn swaps(transaction_results: Vec<TransactionResultMeta>, client: &EnvClient) -> Vec<Swap> {
    transaction_results
        .iter()
        .filter(is_transaction_successful)
        .flat_map(|txn| swaps_from_transaction(txn, client))
        .collect()
}

fn is_transaction_successful(transaction: &&TransactionResultMeta) -> bool {
    matches!(
        transaction.result.result.result,
        TransactionResultResult::TxSuccess(_)
    )
}

fn swaps_from_transaction(transaction: &TransactionResultMeta, client: &EnvClient) -> Vec<Swap> {
    let operations = extract_transaction_results(transaction);
    operations
        .iter()
        .flat_map(|op| swaps_from_operation(op, client))
        .collect()
}

fn swaps_from_operation(operation: &OperationResultTr, client: &EnvClient) -> Vec<Swap> {
    match operation {
        OperationResultTr::ManageSellOffer(ManageSellOfferResult::Success(offer_result))
        | OperationResultTr::CreatePassiveSellOffer(ManageSellOfferResult::Success(offer_result))
        | OperationResultTr::ManageBuyOffer(ManageBuyOfferResult::Success(offer_result)) => {
            swap_from_offer(offer_result, client)
        }
        OperationResultTr::PathPaymentStrictReceive(PathPaymentStrictReceiveResult::Success(_))
        | OperationResultTr::PathPaymentStrictSend(PathPaymentStrictSendResult::Success(_)) => {
            swaps_from_path_payment(operation, client)
        }
        _ => Vec::new(),
    }
}

fn swap_from_offer(offer_result: &ManageOfferSuccessResult, client: &EnvClient) -> Vec<Swap> {
    match &offer_result.offer {
        ManageOfferSuccessResultOffer::Created(offer_entry)
        | ManageOfferSuccessResultOffer::Updated(offer_entry)
            if offer_entry.selling == USDC || offer_entry.buying == USDC =>
        {
            vec![Swap::from(offer_entry.clone())]
        }
        _ => Vec::new(),
    }
}

fn swaps_from_path_payment(operation_result: &OperationResultTr, client: &EnvClient) -> Vec<Swap> {
    let claim_atoms = extract_claim_atoms_from_path_payment_result(operation_result);

    claim_atoms
        .iter()
        // If try_from returns an error, that's because no USDC was involved;
        // we don't want to include that swap.
        .filter_map(|claim_atom| Swap::try_from(claim_atom).ok())
        .collect()
}
