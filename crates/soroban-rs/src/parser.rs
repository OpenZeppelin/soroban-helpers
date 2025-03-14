use crate::error::SorobanHelperError;
use stellar_rpc_client::GetTransactionResponse;
use stellar_xdr::curr::{
    AccountEntry, LedgerEntryChange, LedgerEntryData, OperationResult, ScVal, TransactionMeta,
    TransactionResultResult,
};

#[derive(Debug)]
pub enum ParserType {
    AccountSetOptions,
    InvokeFunction,
    Deploy,
    // Add more parser types as needed
}

#[derive(Debug)]
pub enum ParseResult {
    AccountSetOptions(Option<AccountEntry>),
    InvokeFunction(Option<ScVal>),
    Deploy(Option<ScVal>),
    // Add more result types as needed
}

pub struct Parser {
    parser_type: ParserType,
}

impl Parser {
    pub fn new(parser_type: ParserType) -> Self {
        Self { parser_type }
    }

    pub fn parse(
        &self,
        response: &GetTransactionResponse,
    ) -> Result<ParseResult, SorobanHelperError> {
        match self.parser_type {
            ParserType::AccountSetOptions => {
                self.check_tx_success(&response.result)?;

                // Extract account entry from transaction metadata
                let result = response
                    .result_meta
                    .as_ref()
                    .and_then(|meta| self.extract_account_entry(meta));

                Ok(ParseResult::AccountSetOptions(result))
            }
            ParserType::InvokeFunction => {
                let op_results = self.check_tx_success(&response.result)?;

                // Try to extract return value from transaction metadata first
                let result_from_meta = response
                    .result_meta
                    .as_ref()
                    .and_then(|meta| self.extract_return_value(meta))
                    .map(|value| {
                        ParseResult::InvokeFunction(Some(value))
                    });
                if let Some(result) = result_from_meta {
                    return Ok(result);
                }

                let result_from_op_results = op_results
                    .first()
                    .and_then(|op| self.extract_operation_result(op))
                    .map(|value| {
                        ParseResult::InvokeFunction(Some(value))
                    });
                if let Some(result) = result_from_op_results {
                    return Ok(result);
                }

                // If we couldn't extract a valid result but transaction succeeded
                Ok(ParseResult::InvokeFunction(None))
            }
            ParserType::Deploy => todo!(),
        }
    }

    fn check_tx_success<'a>(&self, tx_result: &'a Option<stellar_xdr::curr::TransactionResult>) -> Result<&'a [OperationResult], SorobanHelperError> {
        let tx_result = tx_result.as_ref().ok_or_else(|| {
            SorobanHelperError::TransactionFailed(
                "No transaction result available".to_string(),
            )
        })?;

        match &tx_result.result {
            TransactionResultResult::TxSuccess(results) => Ok(results.as_slice()),
            _ => Err(SorobanHelperError::TransactionFailed(format!(
                "Transaction failed: {:?}",
                tx_result.result
            ))),
        }
    }

    fn extract_account_entry(&self, meta: &TransactionMeta) -> Option<AccountEntry> {
        match meta {
            TransactionMeta::V3(v3) => v3.operations.last().and_then(|op| {
                op.changes.0.iter().rev().find_map(|change| match change {
                    LedgerEntryChange::Updated(entry) => {
                        if let LedgerEntryData::Account(account) = &entry.data {
                            Some(account.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
            }),
            _ => None,
        }
    }

    fn extract_return_value(&self, meta: &TransactionMeta) -> Option<ScVal> {
        match meta {
            TransactionMeta::V3(v3) => v3.soroban_meta.as_ref().map(|sm| sm.return_value.clone()),
            _ => None,
        }
    }

    fn extract_operation_result(&self, op_result: &OperationResult) -> Option<ScVal> {
        match op_result {
            OperationResult::OpInner(stellar_xdr::curr::OperationResultTr::InvokeHostFunction(
                stellar_xdr::curr::InvokeHostFunctionResult::Success(value),
            )) => Some(ScVal::Symbol(stellar_xdr::curr::ScSymbol(
                value.0.to_vec().try_into().unwrap_or_default(),
            ))),
            _ => None,
        }
    }
}
