use ed25519_dalek::{SigningKey, ed25519::signature::SignerMut};
use sha2::{Digest, Sha256};
use stellar_strkey::ed25519::PublicKey;
use stellar_xdr::curr::{
    AccountId, DecoratedSignature, Hash, Limits, PublicKey as XDRPublicKey, Signature,
    SignatureHint, Transaction, TransactionSignaturePayload,
    TransactionSignaturePayloadTaggedTransaction, WriteXdr,
};
use crate::error::SorobanHelperError;

pub struct Signer {
    signing_key: SigningKey,
    public_key: PublicKey,
    account_id: AccountId,
}

impl Signer {
    pub fn new(signing_key: SigningKey) -> Self {
        let public_key = PublicKey(*signing_key.verifying_key().as_bytes());
        let account_id = AccountId(XDRPublicKey::PublicKeyTypeEd25519(public_key.0.into()));

        Self {
            signing_key,
            public_key,
            account_id,
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn account_id(&self) -> AccountId {
        self.account_id.clone()
    }

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
            signature_payload.to_xdr(Limits::none())
                .map_err(|e| SorobanHelperError::XdrEncodingFailed(e.to_string()))?
        ).into();

        let hint = SignatureHint(
            self.signing_key.verifying_key().to_bytes()[28..].try_into()
                .map_err(|_| SorobanHelperError::SigningFailed("Failed to create signature hint".to_string()))?
        );

        let signature = Signature(
            self.signing_key
                .clone()
                .sign(&tx_hash)
                .to_bytes()
                .to_vec()
                .try_into()
                .map_err(|_| SorobanHelperError::SigningFailed("Failed to convert signature to XDR".to_string()))?
        );

       Ok(DecoratedSignature { hint, signature })
    }
}
