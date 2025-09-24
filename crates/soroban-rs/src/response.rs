use stellar_rpc_client::GetTransactionResponse;
use stellar_xdr::curr::{ScVal, SorobanTransactionMeta, TransactionMeta, TransactionMetaV3};

use crate::SorobanHelperError;

/// Extended transaction response with methods to extract Soroban-specific data
#[derive(Debug, Clone)]
pub struct SorobanTransactionResponse {
    /// The underlying RPC transaction response
    pub response: GetTransactionResponse,
}

impl From<GetTransactionResponse> for SorobanTransactionResponse {
    fn from(response: GetTransactionResponse) -> Self {
        Self { response }
    }
}

impl SorobanTransactionResponse {
    /// Creates a new SorobanTransactionResponse from a GetTransactionResponse
    pub fn new(response: GetTransactionResponse) -> Self {
        Self { response }
    }

    /// Extracts the Soroban transaction return value from the transaction metadata
    ///
    /// # Returns
    ///
    /// The Soroban return value as an ScVal or an error if:
    /// - The transaction result is not available
    /// - The transaction metadata is not available
    /// - The transaction metadata is not in V3 format
    /// - The Soroban metadata is not available
    pub fn get_return_value(&self) -> Result<ScVal, SorobanHelperError> {
        // Check if result_meta exists
        let result_meta = self.response.result_meta.as_ref().ok_or_else(|| {
            SorobanHelperError::InvalidArgument("Transaction metadata not available".to_string())
        })?;

        // Extract the Soroban metadata from the transaction metadata
        match result_meta {
            TransactionMeta::V3(meta_v3) => self.extract_soroban_return_value(meta_v3),
            _ => Err(SorobanHelperError::InvalidArgument(
                "Transaction metadata is not in V3 format (not a Soroban transaction)".to_string(),
            )),
        }
    }

    /// Extracts the Soroban transaction events from the transaction metadata
    ///
    /// # Returns
    ///
    /// A vector of contract events or an error if:
    /// - The transaction result is not available
    /// - The transaction metadata is not available
    /// - The transaction metadata is not in V3 format
    /// - The Soroban metadata is not available
    pub fn get_events(&self) -> Result<Vec<stellar_xdr::curr::ContractEvent>, SorobanHelperError> {
        // Check if result_meta exists
        let result_meta = self.response.result_meta.as_ref().ok_or_else(|| {
            SorobanHelperError::InvalidArgument("Transaction metadata not available".to_string())
        })?;

        // Extract the Soroban metadata from the transaction metadata
        match result_meta {
            TransactionMeta::V3(meta_v3) => self.extract_soroban_events(meta_v3),
            _ => Err(SorobanHelperError::InvalidArgument(
                "Transaction metadata is not in V3 format (not a Soroban transaction)".to_string(),
            )),
        }
    }

    /// Helper method to extract the Soroban return value from a TransactionMetaV3
    fn extract_soroban_return_value(
        &self,
        meta_v3: &TransactionMetaV3,
    ) -> Result<ScVal, SorobanHelperError> {
        // Extract the Soroban metadata
        let soroban_meta = meta_v3.soroban_meta.as_ref().ok_or_else(|| {
            SorobanHelperError::InvalidArgument("Soroban metadata not available".to_string())
        })?;

        // Return a clone of the return value
        Ok(soroban_meta.return_value.clone())
    }

    /// Helper method to extract the Soroban events from a TransactionMetaV3
    fn extract_soroban_events(
        &self,
        meta_v3: &TransactionMetaV3,
    ) -> Result<Vec<stellar_xdr::curr::ContractEvent>, SorobanHelperError> {
        // Extract the Soroban metadata
        let soroban_meta = meta_v3.soroban_meta.as_ref().ok_or_else(|| {
            SorobanHelperError::InvalidArgument("Soroban metadata not available".to_string())
        })?;

        // Convert the VecM to a Vec
        let events = soroban_meta.events.to_vec();
        Ok(events)
    }

    /// Extracts the full Soroban transaction metadata
    ///
    /// # Returns
    ///
    /// The Soroban transaction metadata or an error if:
    /// - The transaction result is not available
    /// - The transaction metadata is not available
    /// - The transaction metadata is not in V3 format
    /// - The Soroban metadata is not available
    pub fn get_soroban_meta(&self) -> Result<SorobanTransactionMeta, SorobanHelperError> {
        // Check if result_meta exists
        let result_meta = self.response.result_meta.as_ref().ok_or_else(|| {
            SorobanHelperError::InvalidArgument("Transaction metadata not available".to_string())
        })?;

        // Extract the Soroban metadata from the transaction metadata
        match result_meta {
            TransactionMeta::V3(meta_v3) => {
                // Extract the Soroban metadata
                let soroban_meta = meta_v3.soroban_meta.as_ref().ok_or_else(|| {
                    SorobanHelperError::InvalidArgument(
                        "Soroban metadata not available".to_string(),
                    )
                })?;

                // Return a clone of the Soroban metadata
                Ok(soroban_meta.clone())
            }
            _ => Err(SorobanHelperError::InvalidArgument(
                "Transaction metadata is not in V3 format (not a Soroban transaction)".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mock::create_mock_contract_event;

    use super::*;
    use stellar_xdr::curr::{
        ContractEvent, ExtensionPoint, LedgerEntryChanges, SorobanTransactionMetaExt,
        TransactionResult, TransactionResultExt, TransactionResultResult, VecM,
    };

    #[test]
    fn test_get_return_value_success() {
        // Create a mock GetTransactionResponse with a V3 transaction meta
        let response = create_mock_response(Some(ScVal::U32(42)));
        let soroban_response = SorobanTransactionResponse::new(response);

        // Test extracting the return value
        let return_value = soroban_response.get_return_value().unwrap();
        assert_eq!(return_value, ScVal::U32(42));
    }

    #[test]
    fn test_get_return_value_no_meta() {
        // Create a mock GetTransactionResponse with no transaction meta
        let response = GetTransactionResponse {
            status: "success".to_string(),
            envelope: None,
            result: None,
            result_meta: None,
            ledger: None,
            events: stellar_rpc_client::GetTransactionEvents {
                contract_events: vec![],
                diagnostic_events: vec![],
                transaction_events: vec![],
            },
        };
        let soroban_response = SorobanTransactionResponse::new(response);

        // Test extracting the return value - should fail
        let result = soroban_response.get_return_value();
        assert!(result.is_err());
    }

    // Helper function to create a mock GetTransactionResponse
    fn create_mock_response(return_value: Option<ScVal>) -> GetTransactionResponse {
        // Create a mock Soroban transaction meta
        let soroban_meta = SorobanTransactionMeta {
            ext: SorobanTransactionMetaExt::V0,
            events: VecM::default(),
            return_value: return_value.unwrap_or(ScVal::Void),
            diagnostic_events: VecM::default(),
        };

        // Create a mock V3 transaction meta
        let meta_v3 = TransactionMetaV3 {
            ext: ExtensionPoint::V0,
            tx_changes_before: LedgerEntryChanges::default(),
            operations: VecM::default(),
            tx_changes_after: LedgerEntryChanges::default(),
            soroban_meta: Some(soroban_meta),
        };

        let transaction_result = TransactionResult {
            fee_charged: 0,
            result: TransactionResultResult::TxSuccess(VecM::default()),
            ext: TransactionResultExt::V0,
        };

        // Create a mock GetTransactionResponse
        GetTransactionResponse {
            status: "success".to_string(),
            envelope: None,
            result: Some(transaction_result),
            result_meta: Some(TransactionMeta::V3(meta_v3)),
            ledger: Some(123456), // Example ledger number
            events: stellar_rpc_client::GetTransactionEvents {
                contract_events: vec![],
                diagnostic_events: vec![],
                transaction_events: vec![],
            },
        }
    }

    #[test]
    fn test_get_events_success() {
        // Create a mock ContractEvent
        let event1 = create_mock_contract_event();
        let event2 = create_mock_contract_event();

        // Create a VecM with the events
        let events: VecM<ContractEvent> = vec![event1.clone(), event2.clone()].try_into().unwrap();

        // Create a mock GetTransactionResponse with events
        let response = create_mock_response_with_events(Some(ScVal::Void), events);
        let soroban_response = SorobanTransactionResponse::new(response);

        // Test extracting the events
        let extracted_events = soroban_response.get_events().unwrap();
        assert_eq!(extracted_events.len(), 2);
    }

    #[test]
    fn test_get_events_no_meta() {
        // Create a mock GetTransactionResponse with no transaction meta
        let response = GetTransactionResponse {
            status: "success".to_string(),
            envelope: None,
            result: None,
            result_meta: None,
            ledger: None,
            events: stellar_rpc_client::GetTransactionEvents {
                contract_events: vec![],
                diagnostic_events: vec![],
                transaction_events: vec![],
            },
        };
        let soroban_response = SorobanTransactionResponse::new(response);

        // Test extracting the events - should fail
        let result = soroban_response.get_events();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_events_empty() {
        // Create a mock GetTransactionResponse with empty events
        let response = create_mock_response(Some(ScVal::Void));
        let soroban_response = SorobanTransactionResponse::new(response);

        // Test extracting the events - should return empty vector
        let events = soroban_response.get_events().unwrap();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_get_soroban_meta_success() {
        // Create a mock GetTransactionResponse
        let return_value = ScVal::U32(42);
        let response = create_mock_response(Some(return_value.clone()));
        let soroban_response = SorobanTransactionResponse::new(response);

        // Test extracting the Soroban metadata
        let soroban_meta = soroban_response.get_soroban_meta().unwrap();

        // Verify the metadata contents
        assert_eq!(soroban_meta.return_value, return_value);
        assert_eq!(soroban_meta.events.len(), 0);
        assert_eq!(soroban_meta.diagnostic_events.len(), 0);
        assert!(matches!(soroban_meta.ext, SorobanTransactionMetaExt::V0));
    }

    #[test]
    fn test_get_soroban_meta_no_meta() {
        // Create a mock GetTransactionResponse with no transaction meta
        let response = GetTransactionResponse {
            status: "success".to_string(),
            envelope: None,
            result: None,
            result_meta: None,
            ledger: None,
            events: stellar_rpc_client::GetTransactionEvents {
                contract_events: vec![],
                diagnostic_events: vec![],
                transaction_events: vec![],
            },
        };
        let soroban_response = SorobanTransactionResponse::new(response);

        // Test extracting the Soroban metadata - should fail
        let result = soroban_response.get_soroban_meta();
        assert!(result.is_err());
    }

    // Helper function to create a mock GetTransactionResponse with custom events
    fn create_mock_response_with_events(
        return_value: Option<ScVal>,
        events: VecM<stellar_xdr::curr::ContractEvent>,
    ) -> GetTransactionResponse {
        // Create a mock Soroban transaction meta
        let soroban_meta = SorobanTransactionMeta {
            ext: SorobanTransactionMetaExt::V0,
            events,
            return_value: return_value.unwrap_or(ScVal::Void),
            diagnostic_events: VecM::default(),
        };

        // Create a mock V3 transaction meta
        let meta_v3 = TransactionMetaV3 {
            ext: ExtensionPoint::V0,
            tx_changes_before: LedgerEntryChanges::default(),
            operations: VecM::default(),
            tx_changes_after: LedgerEntryChanges::default(),
            soroban_meta: Some(soroban_meta),
        };

        let transaction_result = TransactionResult {
            fee_charged: 0,
            result: TransactionResultResult::TxSuccess(VecM::default()),
            ext: TransactionResultExt::V0,
        };

        // Create a mock GetTransactionResponse
        GetTransactionResponse {
            status: "success".to_string(),
            envelope: None,
            result: Some(transaction_result),
            result_meta: Some(TransactionMeta::V3(meta_v3)),
            ledger: Some(123456), // Example ledger number
            events: stellar_rpc_client::GetTransactionEvents {
                contract_events: vec![],
                diagnostic_events: vec![],
                transaction_events: vec![],
            },
        }
    }
}
