use stellar_xdr::curr::{OperationResult, ScVal, TransactionMeta, TransactionResultResult};

pub fn extract_return_value(meta: &TransactionMeta) -> Option<ScVal> {
    match meta {
        TransactionMeta::V3(v3) => v3.soroban_meta.as_ref().map(|sm| sm.return_value.clone()),
        _ => None,
    }
}

pub fn extract_operation_result(op_result: &OperationResult) -> Option<ScVal> {
    if let OperationResult::OpInner(stellar_xdr::curr::OperationResultTr::InvokeHostFunction(
        invoke_result,
    )) = op_result
    {
        if let stellar_xdr::curr::InvokeHostFunctionResult::Success(value) = invoke_result {
            return Some(ScVal::Symbol(stellar_xdr::curr::ScSymbol(
                value.0.to_vec().try_into().unwrap_or_default(),
            )));
        }
    }
    None
}

pub fn parse_transaction_result(
    result: &stellar_rpc_client::GetTransactionResponse,
) -> Result<ScVal, Box<dyn std::error::Error>> {
    if let Some(tx_result) = &result.result {
        if let TransactionResultResult::TxSuccess(op_results) = &tx_result.result {
            // First try to get result from transaction metadata
            if let Some(result_meta) = &result.result_meta {
                if let Some(return_value) = extract_return_value(result_meta) {
                    return Ok(return_value);
                }
            }

            // Then try to get from operation results
            if let Some(op_result) = op_results.first() {
                if let Some(value) = extract_operation_result(op_result) {
                    return Ok(value);
                }
            }

            return Ok(ScVal::Void);
        } else {
            return Err(format!("Transaction failed: {:?}", tx_result.result).into());
        }
    } else {
        return Err("No transaction result available".into());
    }
}
