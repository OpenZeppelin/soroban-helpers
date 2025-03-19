//! # Soroban Transaction Signing
//!
//! This module provides functionality for creating signers and signing Soroban transactions.
//! It handles the cryptographic operations required to sign transactions using Ed25519 keys,
//! following the Stellar transaction signing protocol.
//!
//! ## Example
//!
//! ```rust,no_run
//! use soroban_rs::Signer;
//! use ed25519_dalek::SigningKey;
//! use stellar_xdr::curr::{Hash, Transaction};
//!
//! // Create a signer from a signing key
//! let signing_key = SigningKey::from_bytes(&[/* your private key bytes */]);
//! let signer = Signer::new(signing_key);
//!
//! // Get the associated Stellar account ID
//! let account_id = signer.account_id();
//!
//! // Sign a transaction
//! let signature = signer.sign_transaction(&tx, &network_id)?;
//! ```
use crate::error::SorobanHelperError;
use ed25519_dalek::{SigningKey, ed25519::signature::SignerMut};
use sha2::{Digest, Sha256};
use stellar_strkey::ed25519::PublicKey;
use stellar_xdr::curr::{
    AccountId, DecoratedSignature, Hash, Limits, PublicKey as XDRPublicKey, Signature,
    SignatureHint, Transaction, TransactionSignaturePayload,
    TransactionSignaturePayloadTaggedTransaction, WriteXdr,
};

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
