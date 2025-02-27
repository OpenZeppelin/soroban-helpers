use ed25519_dalek::{SigningKey, ed25519::signature::SignerMut};
use sha2::{Digest, Sha256};
use stellar_strkey::ed25519::{PrivateKey, PublicKey};
use stellar_xdr::curr::{
    AccountId, DecoratedSignature, Hash, Limits, PublicKey as XDRPublicKey, Signature,
    SignatureHint, Transaction, TransactionEnvelope, TransactionSignaturePayload,
    TransactionSignaturePayloadTaggedTransaction, TransactionV1Envelope, VecM, WriteXdr,
};

pub struct Signer {
    signing_key: SigningKey,
    public_key: PublicKey,
    account_id: AccountId,
}

impl Signer {
    pub fn new(private_key_string: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let private_key = PrivateKey::from_string(private_key_string)?;
        let signing_key = SigningKey::from_bytes(&private_key.0);
        let public_key = PublicKey(*signing_key.verifying_key().as_bytes());
        let account_id = AccountId(XDRPublicKey::PublicKeyTypeEd25519(public_key.0.into()));

        Ok(Self {
            signing_key,
            public_key,
            account_id,
        })
    }

    pub fn account_id(&self) -> AccountId {
        self.account_id.clone()
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn sign_transaction(
        &self,
        tx: &Transaction,
        network_id: &Hash,
    ) -> Result<TransactionEnvelope, Box<dyn std::error::Error>> {
        let signature_payload = TransactionSignaturePayload {
            network_id: network_id.clone(),
            tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(tx.clone()),
        };

        let tx_hash: [u8; 32] = Sha256::digest(signature_payload.to_xdr(Limits::none())?).into();

        let hint = SignatureHint(self.signing_key.verifying_key().to_bytes()[28..].try_into()?);

        let signature = Signature(
            self.signing_key
                .clone()
                .sign(&tx_hash)
                .to_bytes()
                .to_vec()
                .try_into()?,
        );

        let signatures: VecM<DecoratedSignature, 20> =
            vec![DecoratedSignature { hint, signature }].try_into()?;

        Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: tx.clone(),
            signatures,
        }))
    }
}
