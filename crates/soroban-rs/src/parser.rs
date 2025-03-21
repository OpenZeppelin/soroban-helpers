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
    use super::*;
    use crate::mock::transaction::{
        create_contract_id_val, mock_transaction_response_with_return_value, 
        mock_transaction_response_with_account_entry
    };
    use crate::mock::parser::{
        mock_to_real_response, create_mock_set_options_tx_envelope,
        MockGetTransactionResponse, MockTransactionResult, MockTransactionMeta
    };
    use stellar_xdr::curr::{ScVal, AccountEntry};
    use std::convert::TryInto;
    
    // Test the Deploy parser
    #[test]
    fn test_deploy_parser() {
        let parser = Parser::new(ParserType::Deploy);
        
        // Create a contract ID ScVal
        let contract_val = create_contract_id_val();
        
        // Use new direct mock function
        let direct_response = mock_transaction_response_with_return_value(contract_val.clone());
        match parser.parse(&direct_response) {
            Ok(ParseResult::Deploy(contract_id)) => {
                assert!(contract_id.is_some());
            },
            _ => panic!("Expected Deploy result with contract ID using direct mock"),
        }
        
        // For backward compatibility - also test with the mock conversion approach
        let mock = MockGetTransactionResponse {
            tx_result: Some(MockTransactionResult { success: true }),
            tx_meta: Some(MockTransactionMeta {
                return_value: Some(contract_val),
                account_entry: None,
            }),
            tx_envelope: None,
        };
        
        // Convert mock to real response and parse
        let response = mock_to_real_response(&mock);
        match parser.parse(&response) {
            Ok(ParseResult::Deploy(contract_id)) => {
                assert!(contract_id.is_some());
            },
            _ => panic!("Expected Deploy result with contract ID"),
        }
    }
    
    // Test the InvokeFunction parser
    #[test]
    fn test_invoke_function_parser() {
        let parser = Parser::new(ParserType::InvokeFunction);
        
        // Create return value
        let return_val = ScVal::I32(42);
        
        // Use new direct mock function
        let direct_response = mock_transaction_response_with_return_value(return_val.clone());
        match parser.parse(&direct_response) {
            Ok(ParseResult::InvokeFunction(value)) => {
                assert!(value.is_some());
                assert_eq!(value.unwrap(), return_val);
            },
            _ => panic!("Expected InvokeFunction result with value using direct mock"),
        }
        
        // For backward compatibility - also test with the mock conversion approach
        let mock = MockGetTransactionResponse {
            tx_result: Some(MockTransactionResult { success: true }),
            tx_meta: Some(MockTransactionMeta {
                return_value: Some(return_val.clone()),
                account_entry: None,
            }),
            tx_envelope: None,
        };
        
        // Convert mock to real response and parse
        let response = mock_to_real_response(&mock);
        match parser.parse(&response) {
            Ok(ParseResult::InvokeFunction(value)) => {
                assert!(value.is_some());
                assert_eq!(value.unwrap(), return_val);
            },
            _ => panic!("Expected InvokeFunction result with value"),
        }
    }
    
    // Test the AccountSetOptions parser
    #[test]
    fn test_account_set_options_parser() {
        // Create a mock account entry
        let account_entry = AccountEntry {
            account_id: stellar_xdr::curr::AccountId(
                stellar_xdr::curr::PublicKey::PublicKeyTypeEd25519(
                    stellar_xdr::curr::Uint256([0; 32])
                )
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
        
        let parser = Parser::new(ParserType::AccountSetOptions);
        
        // Use new direct mock function
        let direct_response = mock_transaction_response_with_account_entry(account_entry.clone());
        match parser.parse(&direct_response) {
            Ok(ParseResult::AccountSetOptions(acct)) => {
                assert!(acct.is_some());
                if let Some(a) = acct {
                    assert_eq!(a.balance, 1000);
                }
            },
            _ => panic!("Expected AccountSetOptions result with direct mock"),
        }
        
        // For backward compatibility - also test with the mock conversion approach
        // Get a mock transaction envelope
        let mock_tx_envelope = create_mock_set_options_tx_envelope();
        
        // Create mock with account entry and tx envelope
        let mock = MockGetTransactionResponse {
            tx_result: Some(MockTransactionResult { success: true }),
            tx_meta: Some(MockTransactionMeta {
                return_value: None,
                account_entry: Some(account_entry),
            }),
            tx_envelope: Some(mock_tx_envelope),
        };
        
        // Convert mock to real response and parse
        let response = mock_to_real_response(&mock);
        match parser.parse(&response) {
            Ok(ParseResult::AccountSetOptions(acct)) => {
                assert!(acct.is_some());
                if let Some(a) = acct {
                    assert_eq!(a.balance, 1000);
                }
            },
            _ => panic!("Expected AccountSetOptions result"),
        }
    }
    
    // Simple test for creating a parser
    #[test]
    fn test_new_parser() {
        let parser = Parser::new(ParserType::InvokeFunction);
        assert!(matches!(parser.parser_type, ParserType::InvokeFunction));
    }
}
