mod api;
mod config;
mod db;
mod filter;
mod utils;

use zephyr_sdk::{
    soroban_sdk::xdr::{TransactionEnvelope, TransactionResultMeta},
    EnvClient,
};

#[no_mangle]
pub extern "C" fn on_close() {
    // The basics
    let client = EnvClient::new();

    // Debug
    client
        .log()
        .debug("B2 before calling envelopes_with_meta", None);

    let envelopes_with_meta = client.reader().envelopes_with_meta();

    client
        .log()
        .debug("B2 after calling envelopes_with_meta", None);

    let (envelopes, transaction_results): (Vec<TransactionEnvelope>, Vec<TransactionResultMeta>) =
        envelopes_with_meta.into_iter().unzip();

    let path_payments = filter::tmp_path_payments(&envelopes);

    client
        .log()
        .debug(&format!("TMP: {} path payments", path_payments.len()), None);

    let tmp_path_payments_with_offer_results =
        filter::tmp_pps_with_offer_results(&envelopes, &transaction_results);

    if !tmp_path_payments_with_offer_results.is_empty() {
        client.log().debug(
            format!(
                "Number of path payments which do not give PP results: {}",
                tmp_path_payments_with_offer_results.len()
            ),
            None,
        );
        let (envelope, result) = tmp_path_payments_with_offer_results.first().unwrap();
        let hash = client.reader().txhash_by_transaction(envelope);
        let json = serde_json::json!({
            "hash": hex::encode(hash),
            "envelope": envelope,
            "result": result,
        });

        client
            .log()
            .debug(&format!("Path payment with offer results: {json}"), None)
    }

    // Read and save the swaps from the latest sequence
    // let _swaps_from_path_payment_offers =
    //     filter::swaps_from_path_payment_offers(&envelopes, &transaction_results);
    // let _swaps_from_elsewhere = filter::swaps_from_elsewhere(&envelopes, &transaction_results);

    // db::save_swaps(
    //     &client,
    //     &swaps_from_path_payment_offers
    //         .iter()
    //         .chain(&swaps_from_elsewhere)
    //         .cloned()
    //         .collect::<Vec<_>>(),
    // );

    // If it is time, calculate and save the exchange rates from the latest sequence
    // db::save_rates(&client);
}
