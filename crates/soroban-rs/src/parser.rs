use crate::error::SorobanHelperError;
use stellar_rpc_client::GetTransactionResponse;
use stellar_strkey::Contract as ContractId;
use stellar_xdr::curr::{
    AccountEntry, LedgerEntryChange, LedgerEntryData, OperationResult, ScAddress, ScVal,
    TransactionMeta, TransactionResultResult,
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
    Deploy(Option<ContractId>),
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
                    .map(|value| ParseResult::InvokeFunction(Some(value)));
                if let Some(result) = result_from_meta {
                    return Ok(result);
                }

                let result_from_op_results = op_results
                    .first()
                    .and_then(|op| self.extract_operation_result(op))
                    .map(|value| ParseResult::InvokeFunction(Some(value)));
                if let Some(result) = result_from_op_results {
                    return Ok(result);
                }

                // If we couldn't extract a valid result but transaction succeeded
                Ok(ParseResult::InvokeFunction(None))
            }
            ParserType::Deploy => {
                self.check_tx_success(&response.result)?;

                // Extract contract hash from transaction metadata
                let result = response
                    .result_meta
                    .as_ref()
                    .and_then(|meta| self.extract_return_value(meta))
                    .and_then(|val| self.extract_contract_id(&val))
                    .map(|contract_id| ParseResult::Deploy(Some(contract_id)));

                if let Some(result) = result {
                    return Ok(result);
                }

                // If we couldn't extract a valid result but transaction succeeded
                Ok(ParseResult::Deploy(None))
            }
        }
    }

    fn check_tx_success<'a>(
        &self,
        tx_result: &'a Option<stellar_xdr::curr::TransactionResult>,
    ) -> Result<&'a [OperationResult], SorobanHelperError> {
        let tx_result = tx_result.as_ref().ok_or_else(|| {
            SorobanHelperError::TransactionFailed("No transaction result available".to_string())
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

    fn extract_contract_id(&self, val: &ScVal) -> Option<ContractId> {
        match val {
            ScVal::Address(ScAddress::Contract(hash)) => Some(ContractId(hash.0)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::SorobanHelperError;
    use crate::mock::transaction::{
        create_contract_id_val, mock_transaction_response_with_account_entry,
        mock_transaction_response_with_return_value,
    };
    use crate::parser::{ParseResult, Parser, ParserType};
    use stellar_rpc_client::GetTransactionResponse;
    use stellar_xdr::curr::{
        AccountEntry, InvokeHostFunctionResult, OperationResult, OperationResultTr, ScVal,
        TransactionResult, TransactionResultExt, TransactionResultResult,
    };

    #[test]
    fn test_new_parser() {
        let parser = Parser::new(ParserType::InvokeFunction);
        assert!(matches!(parser.parser_type, ParserType::InvokeFunction));
    }

    #[test]
    fn test_extract_operation_result_success() {
        let hash_data = [42u8; 32];
        let hash = stellar_xdr::curr::Hash(hash_data);

        let parser = Parser::new(ParserType::InvokeFunction);
        let result = parser
            .extract_operation_result(&OperationResult::OpInner(
                OperationResultTr::InvokeHostFunction(InvokeHostFunctionResult::Success(
                    hash.clone(),
                )),
            ))
            .unwrap();

        assert!(matches!(result, ScVal::Symbol(_)));
        if let ScVal::Symbol(symbol) = result {
            assert_eq!(symbol.0.as_slice(), hash_data);
        }
    }
    #[test]
    fn test_extract_operation_result_non_invoke_function() {
        let parser = Parser::new(ParserType::InvokeFunction);

        // non-InvokeHostFunction operation result
        let op_result = OperationResult::OpInner(OperationResultTr::CreateAccount(
            stellar_xdr::curr::CreateAccountResult::Success,
        ));

        let extracted = parser.extract_operation_result(&op_result);
        assert!(extracted.is_none());
    }

    #[test]
    fn test_extract_operation_result_non_success() {
        let parser = Parser::new(ParserType::InvokeFunction);

        // Failed InvokeHostFunction result
        let op_result = OperationResult::OpInner(OperationResultTr::InvokeHostFunction(
            InvokeHostFunctionResult::ResourceLimitExceeded,
        ));

        let extracted = parser.extract_operation_result(&op_result);
        assert!(extracted.is_none());
    }

    #[test]
    fn test_deploy_parser() {
        let parser = Parser::new(ParserType::Deploy);

        let contract_val = create_contract_id_val();
        let res = mock_transaction_response_with_return_value(contract_val.clone());

        let result = parser.parse(&res.response);
        assert!(matches!(result, Ok(ParseResult::Deploy(Some(_)))));
    }

    #[test]
    fn test_invoke_function_parser() {
        let parser = Parser::new(ParserType::InvokeFunction);

        let return_val = ScVal::I32(42);
        let res = mock_transaction_response_with_return_value(return_val.clone());

        let result = parser.parse(&res.response);
        assert!(matches!(result, Ok(ParseResult::InvokeFunction(Some(_)))));
        if let Ok(ParseResult::InvokeFunction(Some(value))) = result {
            assert_eq!(value, return_val);
        }
    }

    #[test]
    fn test_account_set_options_parser() {
        let parser = Parser::new(ParserType::AccountSetOptions);

        // Create a mock account entry
        let account_entry = AccountEntry {
            account_id: stellar_xdr::curr::AccountId(
                stellar_xdr::curr::PublicKey::PublicKeyTypeEd25519(stellar_xdr::curr::Uint256(
                    [0; 32],
                )),
            ),
            balance: 1000,
            seq_num: 123.into(),
            num_sub_entries: 0,
            inflation_dest: None,
            flags: 0,
            home_domain: stellar_xdr::curr::String32(vec![].try_into().unwrap()),
            thresholds: stellar_xdr::curr::Thresholds([0, 0, 0, 0]),
            signers: stellar_xdr::curr::VecM::default(),
            ext: stellar_xdr::curr::AccountEntryExt::V0,
        };
        let response = mock_transaction_response_with_account_entry(account_entry.clone());

        let result = parser.parse(&response);
        assert!(matches!(
            result,
            Ok(ParseResult::AccountSetOptions(Some(_)))
        ));
        if let Ok(ParseResult::AccountSetOptions(Some(acct))) = result {
            assert_eq!(acct.balance, 1000);
        }
    }

    #[test]
    fn test_no_transaction_result() {
        let response = GetTransactionResponse {
            status: "SUCCESS".to_string(),
            envelope: None,
            result: None, // This is what we're testing - no result
            result_meta: None,
        };

        let parser = Parser::new(ParserType::InvokeFunction);
        let result = parser.parse(&response);
        assert!(matches!(
            result,
            Err(SorobanHelperError::TransactionFailed(_))
        ));
        if let Err(SorobanHelperError::TransactionFailed(msg)) = result {
            assert!(msg.contains("No transaction result available"));
        }
    }

    #[test]
    fn test_invoke_function_fallback_to_operation_result() {
        let parser = Parser::new(ParserType::InvokeFunction);

        // Create a transaction with no metadata but with operation results
        // We simulate a successful transaction but with no result_meta
        let response = GetTransactionResponse {
            status: "SUCCESS".to_string(),
            envelope: None,
            result_meta: None,
            result: Some(TransactionResult {
                fee_charged: 100,
                result: TransactionResultResult::TxSuccess(vec![].try_into().unwrap()),
                ext: TransactionResultExt::V0,
            }),
        };

        // Test the fallback code path where an operation result is checked
        // but not found (empty operations)
        let result = parser.parse(&response);
        assert!(matches!(result, Ok(ParseResult::InvokeFunction(None))));
    }

    #[test]
    fn test_extract_contract_id() {
        let parser = Parser::new(ParserType::Deploy);

        let sc_val = create_contract_id_val();

        let result = parser.extract_contract_id(&sc_val);
        assert!(result.is_some());

        let non_contract_val = ScVal::Bool(true);
        assert!(parser.extract_contract_id(&non_contract_val).is_none());
    }

    #[test]
    fn test_deploy_parser_fallback() {
        let parser = Parser::new(ParserType::Deploy);

        let non_contract_val = ScVal::Bool(true);
        let res = mock_transaction_response_with_return_value(non_contract_val);

        let result = parser.parse(&res.response);
        assert!(matches!(result, Ok(ParseResult::Deploy(None))));

        let response_no_meta = GetTransactionResponse {
            status: "SUCCESS".to_string(),
            envelope: None,
            result: Some(TransactionResult {
                fee_charged: 100,
                result: TransactionResultResult::TxSuccess(vec![].try_into().unwrap()),
                ext: TransactionResultExt::V0,
            }),
            result_meta: None,
        };

        let result = parser.parse(&response_no_meta);
        assert!(matches!(result, Ok(ParseResult::Deploy(None))));
    }
}
