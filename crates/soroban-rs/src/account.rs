use crate::{Provider, Signer, error::SorobanHelperError};
use stellar_xdr::curr::{AccountEntry, AccountId, DecoratedSignature, Hash, SequenceNumber, Transaction, TransactionEnvelope, TransactionV1Envelope, VecM};

pub enum Account {
    KeyPair(SingleAccount),
    Multisig(MultisigAccount),
}

pub struct SingleAccount {
    pub account_id: AccountId,
    pub signers: Vec<Signer>,
}

pub struct MultisigAccount {
    pub account_id: AccountId,
    pub signers: Vec<Signer>,
}

impl Account {
    pub fn account(signer: Signer) -> Self {
        Self::KeyPair(SingleAccount {
            account_id: signer.account_id(),
            signers: vec![signer],
        })
    }

    pub fn multisig(account_id: AccountId, signers: Vec<Signer>) -> Self {
        Self::Multisig(MultisigAccount {
            account_id,
            signers,
        })
    }
    
    pub async fn load(&self, provider: &Provider) -> Result<AccountEntry, SorobanHelperError> {
        match self {
            Self::KeyPair(account) => provider.get_account(&account.account_id.to_string()).await,
            Self::Multisig(account) => provider.get_account(&account.account_id.to_string()).await,
        }
    }

    pub fn account_id(&self) -> AccountId {
        match self {
            Self::KeyPair(account) => account.account_id.clone(),
            Self::Multisig(account) => account.account_id.clone(),
        }
    }

    pub async fn get_sequence(&self, provider: &Provider) -> Result<SequenceNumber, SorobanHelperError> {
        let entry = self.load(provider).await?;
        Ok(entry.seq_num)
    }

    pub fn sign_transaction(
        &self,
        tx: &Transaction,
        network_id: &Hash,
    ) -> Result<TransactionEnvelope, SorobanHelperError> {
        let signers = match self {
            Self::KeyPair(account) => &account.signers,
            Self::Multisig(account) => &account.signers,
        };
        let signatures: VecM<DecoratedSignature, 20> = signers
            .iter()
            .map(|signer| 
                signer.sign_transaction(tx, network_id)
            )
            .collect::<Result<Vec<DecoratedSignature>, SorobanHelperError>>()
            .map_err(|e| SorobanHelperError::XdrEncodingFailed(e.to_string()))?
            .try_into()
            .map_err(|_| SorobanHelperError::XdrEncodingFailed("Failed to convert signatures to XDR".to_string()))?;
        Ok(TransactionEnvelope::Tx(
            TransactionV1Envelope {
                tx: tx.clone(),
                signatures,
            }
        ))
    }
}
