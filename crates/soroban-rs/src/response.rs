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
    use super::*;
    use stellar_xdr::curr::{
        ExtensionPoint, LedgerEntryChanges, SorobanTransactionMetaExt, TransactionResult,
        TransactionResultExt, TransactionResultResult, VecM,
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
        }
    }
}
