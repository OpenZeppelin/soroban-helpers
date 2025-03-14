use crate::{Env, Signer, TransactionBuilder, error::SorobanHelperError};
use stellar_strkey::ed25519::PublicKey;
use stellar_xdr::curr::{
    AccountEntry, AccountId, DecoratedSignature, Hash, Operation, OperationBody, SequenceNumber,
    SetOptionsOp, Signer as XdrSigner, SignerKey, Transaction, TransactionEnvelope,
    TransactionV1Envelope, VecM,
};

pub enum Account {
    KeyPair(SingleAccount),
    Multisig(MultisigAccount),
}

impl Clone for Account {
    fn clone(&self) -> Self {
        match self {
            Self::KeyPair(account) => Self::KeyPair(account.clone()),
            Self::Multisig(account) => Self::Multisig(account.clone()),
        }
    }
}

pub struct SingleAccount {
    pub account_id: AccountId,
    pub signers: Vec<Signer>,
    pub authorized_calls: i16,
}

impl Clone for SingleAccount {
    fn clone(&self) -> Self {
        Self {
            account_id: self.account_id.clone(),
            signers: self.signers.clone(),
            authorized_calls: self.authorized_calls,
        }
    }
}

impl Clone for MultisigAccount {
    fn clone(&self) -> Self {
        Self {
            account_id: self.account_id.clone(),
            signers: self.signers.clone(),
            authorized_calls: self.authorized_calls,
        }
    }
}

pub struct MultisigAccount {
    pub account_id: AccountId,
    pub signers: Vec<Signer>,
    pub authorized_calls: i16,
}

pub struct AccountConfig {
    pub master_weight: Option<u32>,
    pub low_threshold: Option<u32>,
    pub med_threshold: Option<u32>,
    pub high_threshold: Option<u32>,
    pub signers: Vec<(PublicKey, u32)>,
}

impl Default for AccountConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountConfig {
    pub fn new() -> Self {
        Self {
            master_weight: None,
            low_threshold: None,
            med_threshold: None,
            high_threshold: None,
            signers: Vec::new(),
        }
    }

    pub fn with_master_weight(mut self, weight: u32) -> Self {
        self.master_weight = Some(weight);
        self
    }

    pub fn with_thresholds(mut self, low: u32, med: u32, high: u32) -> Self {
        self.low_threshold = Some(low);
        self.med_threshold = Some(med);
        self.high_threshold = Some(high);
        self
    }

    pub fn add_signer(mut self, key: PublicKey, weight: u32) -> Self {
        self.signers.push((key, weight));
        self
    }
}

impl Account {
    pub fn single(signer: Signer) -> Self {
        Self::KeyPair(SingleAccount {
            account_id: signer.account_id(),
            signers: vec![signer],
            authorized_calls: i16::MAX,
        })
    }

    pub fn multisig(account_id: AccountId, signers: Vec<Signer>) -> Self {
        Self::Multisig(MultisigAccount {
            account_id,
            signers,
            authorized_calls: i16::MAX,
        })
    }

    pub async fn load(&self, env: &Env) -> Result<AccountEntry, SorobanHelperError> {
        match self {
            Self::KeyPair(account) => env.get_account(&account.account_id.to_string()).await,
            Self::Multisig(account) => env.get_account(&account.account_id.to_string()).await,
        }
    }

    pub fn account_id(&self) -> AccountId {
        match self {
            Self::KeyPair(account) => account.account_id.clone(),
            Self::Multisig(account) => account.account_id.clone(),
        }
    }

    pub fn set_authorized_calls(&mut self, authorized_calls: i16) {
        match self {
            Self::KeyPair(account) => account.authorized_calls = authorized_calls,
            Self::Multisig(account) => account.authorized_calls = authorized_calls,
        }
    }

    pub async fn get_sequence(&self, env: &Env) -> Result<SequenceNumber, SorobanHelperError> {
        let entry = self.load(env).await?;
        Ok(entry.seq_num)
    }

    pub fn sign_transaction_unsafe(
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
            .map(|signer| signer.sign_transaction(tx, network_id))
            .collect::<Result<Vec<DecoratedSignature>, SorobanHelperError>>()
            .map_err(|e| SorobanHelperError::XdrEncodingFailed(e.to_string()))?
            .try_into()
            .map_err(|_| {
                SorobanHelperError::XdrEncodingFailed(
                    "Failed to convert signatures to XDR".to_string(),
                )
            })?;

        Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: tx.clone(),
            signatures,
        }))
    }

    pub fn sign_transaction(
        &mut self,
        tx: &Transaction,
        network_id: &Hash,
    ) -> Result<TransactionEnvelope, SorobanHelperError> {
        let (signers, authorized_calls) = match self {
            Self::KeyPair(account) => (&account.signers, account.authorized_calls),
            Self::Multisig(account) => (&account.signers, account.authorized_calls),
        };

        if authorized_calls < 1 {
            return Err(SorobanHelperError::Unauthorized(
                "Account has reached the max number of authorized calls".to_string(),
            ));
        }

        let signatures: VecM<DecoratedSignature, 20> = signers
            .iter()
            .map(|signer| signer.sign_transaction(tx, network_id))
            .collect::<Result<Vec<DecoratedSignature>, SorobanHelperError>>()
            .map_err(|e| SorobanHelperError::XdrEncodingFailed(e.to_string()))?
            .try_into()
            .map_err(|_| {
                SorobanHelperError::XdrEncodingFailed(
                    "Failed to convert signatures to XDR".to_string(),
                )
            })?;

        self.set_authorized_calls(authorized_calls - 1);

        Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: tx.clone(),
            signatures,
        }))
    }

    pub fn sign_transaction_envelope(
        &mut self,
        tx: &TransactionEnvelope,
        network_id: &Hash,
    ) -> Result<TransactionEnvelope, SorobanHelperError> {
        let (signers, authorized_calls) = match self {
            Self::KeyPair(account) => (&account.signers, account.authorized_calls),
            Self::Multisig(account) => (&account.signers, account.authorized_calls),
        };

        if authorized_calls < 1 {
            return Err(SorobanHelperError::Unauthorized(
                "Account has reached the max number of authorized calls".to_string(),
            ));
        }

        let tx = match tx {
            TransactionEnvelope::Tx(tx) => tx,
            _ => {
                return Err(SorobanHelperError::XdrEncodingFailed(
                    "Invalid transaction envelope".to_string(),
                ));
            }
        };
        let prev_signatures = tx.signatures.clone();
        let new_signatures: VecM<DecoratedSignature, 20> = signers
            .iter()
            .map(|signer| signer.sign_transaction(&tx.tx, network_id))
            .collect::<Result<Vec<DecoratedSignature>, SorobanHelperError>>()
            .map_err(|e| SorobanHelperError::XdrEncodingFailed(e.to_string()))?
            .try_into()
            .map_err(|_| {
                SorobanHelperError::XdrEncodingFailed(
                    "Failed to convert signatures to XDR".to_string(),
                )
            })?;

        // Convert VecM to Vec, combine with previous signatures, and convert back to VecM
        let mut all_signatures: Vec<DecoratedSignature> = prev_signatures.to_vec();
        all_signatures.extend(new_signatures.to_vec());

        let signatures: VecM<DecoratedSignature, 20> = all_signatures.try_into().map_err(|_| {
            SorobanHelperError::XdrEncodingFailed("Too many signatures for XDR vector".to_string())
        })?;

        self.set_authorized_calls(authorized_calls - 1);

        Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: tx.tx.clone(),
            signatures,
        }))
    }

    pub async fn configure(
        &mut self,
        env: &Env,
        config: AccountConfig,
    ) -> Result<TransactionEnvelope, SorobanHelperError> {
        let account_entry = self.load(env).await?;
        let sequence_num = account_entry.seq_num.0;

        let mut tx = TransactionBuilder::new(self.account_id(), sequence_num + 1);

        // Add set options operation for each configuration item
        if !config.signers.is_empty() {
            for (public_key, weight) in config.signers {
                let signer_key = SignerKey::Ed25519(public_key.0.into());
                tx = tx.add_operation(Operation {
                    source_account: None,
                    body: OperationBody::SetOptions(SetOptionsOp {
                        inflation_dest: None,
                        clear_flags: None,
                        set_flags: None,
                        master_weight: None,
                        low_threshold: None,
                        med_threshold: None,
                        high_threshold: None,
                        home_domain: None,
                        signer: Some(XdrSigner {
                            key: signer_key,
                            weight,
                        }),
                    }),
                });
            }
        }

        // Add thresholds if specified
        if config.master_weight.is_some()
            || config.low_threshold.is_some()
            || config.med_threshold.is_some()
            || config.high_threshold.is_some()
        {
            tx = tx.add_operation(Operation {
                source_account: None,
                body: OperationBody::SetOptions(SetOptionsOp {
                    inflation_dest: None,
                    clear_flags: None,
                    set_flags: None,
                    master_weight: config.master_weight,
                    low_threshold: config.low_threshold,
                    med_threshold: config.med_threshold,
                    high_threshold: config.high_threshold,
                    home_domain: None,
                    signer: None,
                }),
            });
        }

        let tx = tx
            .simulate_and_build(env, self)
            .await
            .map_err(|e| SorobanHelperError::TransactionBuildFailed(e.to_string()))?;

        self.sign_transaction(&tx, &env.network_id())
    }
}
