use crate::pb::eth::transaction::v1::{Transaction, Transactions};
use crate::abi;
use substreams::Hex;
use substreams_ethereum::block_view::CallView;
use substreams_ethereum::pb::eth::v2::{Block, TransactionTrace};

// #[derive(Deserialize)]
struct TransactionFilters {
    filters: Vec<TransactionFilter>
}
struct TransactionFilter {
    original_contract_address_to_skip: String,
    call_signatures_pattern: Vec<String>,
}

#[substreams::handlers::map]
fn map_filter_transactions(blk: Block) -> Result<Transactions, Vec<substreams::errors::Error>> {
    let filters = compose_filters();
    let header = blk.header.unwrap();

    let transactions: Vec<Transaction> = blk
        .transaction_traces.iter()
        .filter_map(|trans| {
            let aa_trans_type = filter_and_get_aa_type(&trans, &filters);
            if aa_trans_type.is_some() {
                Some(Transaction {
                    from: Hex::encode(&trans.from),
                    to: Hex::encode(&trans.to),
                    hash: Hex::encode(&trans.hash),
                    chain: "ethereum".to_owned(),
                    account_abstraction_type: aa_trans_type.unwrap(),
                    status: trans.status().as_str_name().to_owned(),
                    timestamp: Some(header.timestamp.as_ref().unwrap().clone())
                })
            } else {
                None
            }
        })
        .collect();

    Ok(Transactions { transactions })
}

fn compose_filters() -> TransactionFilters {
    let pattern = "handleOps -> simulateValidation";

    let erc4337_filter = TransactionFilter {
        original_contract_address_to_skip: "0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789".to_string(),
        call_signatures_pattern: pattern.split(" -> ")
                                        .map(|s| s.to_string())
                                        .collect()
    };

    let filters = TransactionFilters {
        filters: vec![erc4337_filter]
    };
    
    return filters;
}

fn filter_and_get_aa_type(transaction: &TransactionTrace, filters: &TransactionFilters) -> Option<String> {
    let hex_transaction_to = format!("0x{}", Hex::encode(&transaction.to));
    let mut pass = false;

    for filter in &filters.filters {
        if filter.original_contract_address_to_skip.to_lowercase() == hex_transaction_to {
            return None;    // we shouldn't add the transactions to the original contract, but only the similar ones 
        } else {
            pass = filter.call_signatures_pattern.iter().all(|signature| call_signature_filter(&transaction, &signature));
        }
    }
    if pass {
        return Some("erc4337".to_string());
    } else {
        return None;
    }
}

fn call_signature_filter(trx_trace: &TransactionTrace, signature: &String) -> bool {
    return trx_trace.calls().any(|call| call_signature_match(&call, signature))
}

fn call_signature_match(call: &CallView, signature: &String) -> bool {
    match signature.as_str() {
        "handleOps" => {
            return abi::entrypoint::functions::HandleOps::match_call(&call.call);
        }
        "innerHandleOp" => {
            return abi::entrypoint::functions::InnerHandleOp::match_call(&call.call);
        }
        "simulateValidation" => {
            return abi::entrypoint::functions::SimulateValidation::match_call(&call.call);
            
        }
        "addStake" => {
            return abi::entrypoint::functions::AddStake::match_call(&call.call);
            
        }
        _ => return false
    }
}