use stellar_rpc_client::GetTransactionResponse;
use stellar_xdr::curr::{AccountEntry, LedgerEntryChange, LedgerEntryData, OperationResult, ScVal, TransactionMeta, TransactionResultResult};
use crate::error::SorobanHelperError;

#[derive(Debug)]
pub enum ParserType {
    AccountSetOptions,
    // Add more parser types as needed
}

#[derive(Debug)]
pub enum ParseResult {
    AccountSetOptions(AccountSetOptionsResult),
    // Add more result types as needed
}

#[derive(Debug)]
pub struct AccountSetOptionsResult {
    pub success: bool,
    pub result: Option<AccountEntry>,
}

pub struct Parser {
    parser_type: ParserType,
}

impl Parser {
    pub fn new(parser_type: ParserType) -> Self {
        Self { parser_type }
    }

    pub fn parse(&self, response: &GetTransactionResponse) -> Result<ParseResult, SorobanHelperError> {
        match self.parser_type {
            ParserType::AccountSetOptions => {
                let success = response.result
                    .as_ref()
                    .map(|r| matches!(&r.result, TransactionResultResult::TxSuccess(_)))
                    .unwrap_or(false);

                let result = response.result_meta
                    .as_ref()
                    .and_then(|meta| match meta {
                        TransactionMeta::V3(v3) => v3.operations.last()
                            .and_then(|op| op.changes.0.iter().rev()
                                .find_map(|change| match change {
                                    LedgerEntryChange::Updated(entry) => {
                                        if let LedgerEntryData::Account(account) = &entry.data {
                                            Some(account.clone())
                                        } else {
                                            None
                                        }
                                    },
                                    _ => None,
                                })),
                        _ => None,
                    });

                Ok(ParseResult::AccountSetOptions(AccountSetOptionsResult { success, result }))
            }
        }
    }
}


pub fn extract_return_value(meta: &TransactionMeta) -> Option<ScVal> {
    match meta {
        TransactionMeta::V3(v3) => v3.soroban_meta.as_ref().map(|sm| sm.return_value.clone()),
        _ => None,
    }
}

pub fn extract_operation_result(op_result: &OperationResult) -> Option<ScVal> {
    if let OperationResult::OpInner(stellar_xdr::curr::OperationResultTr::InvokeHostFunction(
        stellar_xdr::curr::InvokeHostFunctionResult::Success(value),
    )) = op_result
    {
        return Some(ScVal::Symbol(stellar_xdr::curr::ScSymbol(
            value.0.to_vec().try_into().unwrap_or_default(),
        )));
    }
    None
}

pub fn parse_transaction_result(
    result: &stellar_rpc_client::GetTransactionResponse,
) -> Result<ScVal, SorobanHelperError> {
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

            Ok(ScVal::Void)
        } else {
            Err(SorobanHelperError::TransactionFailed(format!(
                "Transaction failed: {:?}",
                tx_result.result
            )))
        }
    } else {
        Err(SorobanHelperError::TransactionFailed(
            "No transaction result available".to_string(),
        ))
    }
}
