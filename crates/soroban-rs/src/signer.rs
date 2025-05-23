//! # Soroban Transaction Signing
//!
//! This module provides functionality for creating signers and signing Soroban transactions.
//! It handles the cryptographic operations required to sign transactions using Ed25519 keys,
//! following the Stellar transaction signing protocol.
//!
//! ## Example
//!
//! ```rust,no_run
//! use soroban_rs::{Env, Signer};
//! use ed25519_dalek::SigningKey;
//! use stellar_xdr::curr::Transaction;
//!
//! async fn example(tx: Transaction, env: Env) {
//!     // Create a signer from a signing key
//!     let private_key_bytes: [u8; 32] = [
//!         1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
//!         26, 27, 28, 29, 30, 31, 32,
//!     ];
//!     let signing_key = SigningKey::from_bytes(&private_key_bytes);
//!     let signer = Signer::new(signing_key);
//!
//!     // Get the associated Stellar account ID
//!     let account_id = signer.account_id();
//!
//!     // Sign a transaction
//!     let signature = signer.sign_transaction(&tx, &env.network_id()).unwrap();
//! }
//! ```
use crate::error::SorobanHelperError;
use ed25519_dalek::{ed25519::signature::SignerMut, SigningKey};
use sha2::{Digest, Sha256};
use stellar_strkey::ed25519::PublicKey;
use stellar_xdr::curr::{
    AccountId, DecoratedSignature, Hash, Limits, PublicKey as XDRPublicKey, Signature,
    SignatureHint, Transaction, TransactionSignaturePayload,
    TransactionSignaturePayloadTaggedTransaction, WriteXdr,
};

impl From<&[u8; 32]> for Signer {
    fn from(bytes: &[u8; 32]) -> Self {
        Signer::new(SigningKey::from_bytes(bytes))
    }
}

/// A transaction signer for Soroban operations.
///
/// The Signer manages an Ed25519 key pair and provides methods to sign Stellar transactions.
/// It handles the conversion between various key formats used in the Stellar ecosystem
/// and implements the Stellar transaction signing protocol.
#[derive(Clone)]
pub struct Signer {
    /// The Ed25519 signing key (private key)
    signing_key: SigningKey,
    /// The corresponding public key in Stellar format
    public_key: PublicKey,
    /// The Stellar account ID derived from the public key
    account_id: AccountId,
}

impl Signer {
    /// Creates a new signer from an Ed25519 signing key.
    ///
    /// # Parameters
    ///
    /// * `signing_key` - The Ed25519 signing key (private key)
    ///
    /// # Returns
    ///
    /// A new Signer instance
    pub fn new(signing_key: SigningKey) -> Self {
        let public_key = PublicKey(*signing_key.verifying_key().as_bytes());
        let account_id = AccountId(XDRPublicKey::PublicKeyTypeEd25519(public_key.0.into()));

        Self {
            signing_key,
            public_key,
            account_id,
        }
    }

    /// Returns the public key associated with this signer.
    ///
    /// # Returns
    ///
    /// The Stellar public key
    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    /// Returns the Stellar account ID associated with this signer.
    ///
    /// # Returns
    ///
    /// The Stellar account ID
    pub fn account_id(&self) -> AccountId {
        self.account_id.clone()
    }

    /// Signs a transaction with this signer's private key.
    ///
    /// # Parameters
    ///
    /// * `tx` - The transaction to sign
    /// * `network_id` - The network ID hash
    ///
    /// # Returns
    ///
    /// A decorated signature that can be attached to the transaction
    ///
    /// # Errors
    ///
    /// Returns:
    /// - `SorobanHelperError::XdrEncodingFailed` if the transaction payload cannot be encoded
    /// - `SorobanHelperError::SigningFailed` if there is an error creating the signature
    pub fn sign_transaction(
        &self,
        tx: &Transaction,
        network_id: &Hash,
    ) -> Result<DecoratedSignature, SorobanHelperError> {
        let signature_payload = TransactionSignaturePayload {
            network_id: network_id.clone(),
            tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(tx.clone()),
        };

        let tx_hash: [u8; 32] = Sha256::digest(
            signature_payload
                .to_xdr(Limits::none())
                .map_err(|e| SorobanHelperError::XdrEncodingFailed(e.to_string()))?,
        )
        .into();

        let hint = SignatureHint(
            self.signing_key.verifying_key().to_bytes()[28..]
                .try_into()
                .map_err(|_| {
                    SorobanHelperError::SigningFailed("Failed to create signature hint".to_string())
                })?,
        );

        let signature = Signature(
            self.signing_key
                .clone()
                .sign(&tx_hash)
                .to_bytes()
                .to_vec()
                .try_into()
                .map_err(|_| {
                    SorobanHelperError::SigningFailed(
                        "Failed to convert signature to XDR".to_string(),
                    )
                })?,
        );

        Ok(DecoratedSignature { hint, signature })
    }
}

#[cfg(test)]
mod tests {
    use stellar_xdr::curr::BytesM;

    use crate::mock::mock_transaction;

    use super::*;

    #[test]
    fn test_public_key() {
        let signing_key = SigningKey::from_bytes(&[42; 32]);
        let public_key = PublicKey(*signing_key.verifying_key().as_bytes());

        let signer = Signer::new(signing_key);
        assert_eq!(signer.public_key(), public_key);
    }

    #[test]
    fn test_account_id() {
        let signing_key = SigningKey::from_bytes(&[42; 32]);
        let public_key = PublicKey(*signing_key.verifying_key().as_bytes());
        let account_id = AccountId(XDRPublicKey::PublicKeyTypeEd25519(public_key.0.into()));

        let signer = Signer::new(signing_key);
        assert_eq!(signer.account_id(), account_id);
    }

    #[test]
    fn test_sign_transaction() {
        let signing_key = SigningKey::from_bytes(&[42; 32]);
        let public_key = PublicKey(*signing_key.verifying_key().as_bytes());
        let account_id = AccountId(XDRPublicKey::PublicKeyTypeEd25519(public_key.0.into()));

        let signer = Signer::new(signing_key);

        let transaction = mock_transaction(account_id, vec![]);
        let network_id = Hash::from([42; 32]);

        let decorated_signature = signer.sign_transaction(&transaction, &network_id).unwrap();

        // hex encoded hint
        let hint_vec = hex::decode("3d368d61").expect("Invalid hex");
        let hint: [u8; 4] = hint_vec[..4]
            .try_into()
            .expect("slice with incorrect length");

        // hex encoded signature
        let signature_vec = hex::decode("c84612be60b83b3e13e18880b6f35c94bda449a53103367b78e211f0a7614dc0df02e45539a4879fc37fb908d7983efba2d7019c1ef5732f0c1331b808eec102").expect("Invalid hex");
        let signature_bytes: BytesM<64> = signature_vec
            .try_into()
            .expect("slice with incorrect length");

        assert_eq!(decorated_signature.hint, SignatureHint(hint));
        assert_eq!(decorated_signature.signature, Signature(signature_bytes));
    }
}
