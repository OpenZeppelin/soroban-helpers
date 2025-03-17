use crate::{Env, Signer, TransactionBuilder, error::SorobanHelperError};
use stellar_strkey::ed25519::PublicKey;
use stellar_xdr::curr::{
    AccountEntry, AccountId, DecoratedSignature, Hash, Operation, OperationBody, SetOptionsOp,
    Signer as XdrSigner, SignerKey, Transaction, TransactionEnvelope, TransactionV1Envelope, VecM,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountSequence(i64);

impl AccountSequence {
    pub fn new(val: i64) -> Self {
        AccountSequence(val)
    }

    /// Returns a new SequenceNumber incremented by one.
    pub fn next(&self) -> Self {
        AccountSequence(self.0 + 1)
    }

    /// Consumes self and returns the next SequenceNumber.
    pub fn increment(self) -> Self {
        self.next()
    }

    /// Returns the raw i64 value.
    pub fn value(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AuthorizedCalls(u16);

impl AuthorizedCalls {
    pub fn new(calls: u16) -> Self {
        AuthorizedCalls(calls)
    }

    pub fn can_call(&self) -> bool {
        self.0 > 0
    }

    pub fn try_decrement(&mut self) -> Result<(), SorobanHelperError> {
        if self.can_call() {
            self.0 -= 1;
            Ok(())
        } else {
            Err(SorobanHelperError::Unauthorized(
                "Account has reached the max number of authorized calls".to_string(),
            ))
        }
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

/// Account configuration options.
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

#[derive(Clone)]
pub struct SingleAccount {
    pub account_id: AccountId,
    pub signers: Vec<Signer>,
    pub authorized_calls: AuthorizedCalls,
}

#[derive(Clone)]
pub struct MultisigAccount {
    pub account_id: AccountId,
    pub signers: Vec<Signer>,
    pub authorized_calls: AuthorizedCalls,
}

#[derive(Clone)]
pub enum Account {
    KeyPair(SingleAccount),
    Multisig(MultisigAccount),
}

impl Account {
    pub fn single(signer: Signer) -> Self {
        Self::KeyPair(SingleAccount {
            account_id: signer.account_id(),
            signers: vec![signer],
            authorized_calls: AuthorizedCalls::new(i16::MAX as u16),
        })
    }

    pub fn multisig(account_id: AccountId, signers: Vec<Signer>) -> Self {
        Self::Multisig(MultisigAccount {
            account_id,
            signers,
            authorized_calls: AuthorizedCalls::new(i16::MAX as u16),
        })
    }

    pub async fn load(&self, env: &Env) -> Result<AccountEntry, SorobanHelperError> {
        env.get_account(&self.account_id().to_string()).await
    }

    pub fn account_id(&self) -> AccountId {
        match self {
            Self::KeyPair(account) => account.account_id.clone(),
            Self::Multisig(account) => account.account_id.clone(),
        }
    }

    pub async fn get_sequence(&self, env: &Env) -> Result<AccountSequence, SorobanHelperError> {
        let entry = self.load(env).await?;
        Ok(AccountSequence::new(entry.seq_num.0))
    }

    /// Retrieves the next available sequence number.
    pub async fn next_sequence(&self, env: &Env) -> Result<AccountSequence, SorobanHelperError> {
        let current = self.get_sequence(env).await?;
        Ok(current.next())
    }

    fn signers(&self) -> &[Signer] {
        match self {
            Self::KeyPair(account) => &account.signers,
            Self::Multisig(account) => &account.signers,
        }
    }

    fn authorized_calls(&self) -> AuthorizedCalls {
        match self {
            Self::KeyPair(account) => account.authorized_calls,
            Self::Multisig(account) => account.authorized_calls,
        }
    }

    fn authorized_calls_mut(&mut self) -> &mut AuthorizedCalls {
        match self {
            Self::KeyPair(account) => &mut account.authorized_calls,
            Self::Multisig(account) => &mut account.authorized_calls,
        }
    }

    pub fn set_authorized_calls(&mut self, authorized_calls: i16) {
        *self.authorized_calls_mut() = AuthorizedCalls::new(authorized_calls as u16);
    }

    /// Helper to sign a transaction using the account’s signers.
    fn sign_with_tx(
        tx: &Transaction,
        network_id: &Hash,
        signers: &[Signer],
    ) -> Result<VecM<DecoratedSignature, 20>, SorobanHelperError> {
        let signatures: Vec<DecoratedSignature> = signers
            .iter()
            .map(|signer| signer.sign_transaction(tx, network_id))
            .collect::<Result<_, _>>()
            .map_err(|e| SorobanHelperError::XdrEncodingFailed(e.to_string()))?;
        signatures.try_into().map_err(|_| {
            SorobanHelperError::XdrEncodingFailed("Failed to convert signatures to XDR".to_string())
        })
    }

    /// Signs a transaction without checking or decrementing the authorized_calls counter.
    pub fn sign_transaction_unsafe(
        &self,
        tx: &Transaction,
        network_id: &Hash,
    ) -> Result<TransactionEnvelope, SorobanHelperError> {
        let signatures = Self::sign_with_tx(tx, network_id, self.signers())?;
        Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: tx.clone(),
            signatures,
        }))
    }

    /// Signs a transaction, ensuring the account still has authorized calls.
    pub fn sign_transaction(
        &mut self,
        tx: &Transaction,
        network_id: &Hash,
    ) -> Result<TransactionEnvelope, SorobanHelperError> {
        if !self.authorized_calls().can_call() {
            return Err(SorobanHelperError::Unauthorized(
                "Account has reached the max number of authorized calls".to_string(),
            ));
        }

        let signatures = Self::sign_with_tx(tx, network_id, self.signers())?;
        self.authorized_calls_mut().try_decrement()?;

        Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: tx.clone(),
            signatures,
        }))
    }

    /// Signs a transaction envelope by appending new signatures.
    pub fn sign_transaction_envelope(
        &mut self,
        tx_envelope: &TransactionEnvelope,
        network_id: &Hash,
    ) -> Result<TransactionEnvelope, SorobanHelperError> {
        if !self.authorized_calls().can_call() {
            return Err(SorobanHelperError::Unauthorized(
                "Account has reached the max number of authorized calls".to_string(),
            ));
        }

        let tx_v1 = match tx_envelope {
            TransactionEnvelope::Tx(tx_v1) => tx_v1,
            _ => {
                return Err(SorobanHelperError::XdrEncodingFailed(
                    "Invalid transaction envelope".to_string(),
                ));
            }
        };

        let prev_signatures = tx_v1.signatures.clone();
        let new_signatures = Self::sign_with_tx(&tx_v1.tx, network_id, self.signers())?;

        let mut all_signatures: Vec<DecoratedSignature> = prev_signatures.to_vec();
        all_signatures.extend(new_signatures.to_vec());
        let signatures: VecM<DecoratedSignature, 20> = all_signatures.try_into().map_err(|_| {
            SorobanHelperError::XdrEncodingFailed("Too many signatures for XDR vector".to_string())
        })?;

        self.authorized_calls_mut().try_decrement()?;

        Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: tx_v1.tx.clone(),
            signatures,
        }))
    }

    /// Configures the account by building and signing a transaction that sets options.
    pub async fn configure(
        mut self,
        env: &Env,
        config: AccountConfig,
    ) -> Result<TransactionEnvelope, SorobanHelperError> {
        let mut tx = TransactionBuilder::new(&self, &env);

        // Add set options operation for each signer configuration.
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

        // Add thresholds if any are specified.
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
            .simulate_and_build(env, &self)
            .await
            .map_err(|e| SorobanHelperError::TransactionBuildFailed(e.to_string()))?;

        self.sign_transaction(&tx, &env.network_id())
    }
}

#[cfg(test)]
mod test {
    use stellar_xdr::curr::{OperationBody, Signer as XdrSigner, SignerKey, TransactionEnvelope};

    use crate::account::AuthorizedCalls;
    use crate::mock::mocks::{MockRpcClient, all_signers, mock_signer1, mock_signer3};
    use crate::{Account, AccountConfig, Env, EnvConfigs, TransactionBuilder};
    use std::sync::Arc;

    #[tokio::test]
    async fn load_account() {
        let env_configs = EnvConfigs {
            rpc_url: "https://test.com".to_string(),
            network_passphrase: "Test Network".to_string(),
        };

        let env = Env {
            configs: env_configs,
            rpc_client: Arc::new(MockRpcClient::new()),
        };

        // Test single account operations
        let account = Account::single(mock_signer1());

        // Test account loading
        let entry = account.load(&env).await;

        let expected_account_id = mock_signer1().account_id().0.to_string();
        let res_account_id = entry.unwrap().account_id.0.to_string();

        assert_eq!(expected_account_id, res_account_id);
    }

    #[tokio::test]
    async fn multisig() {
        let env_configs = EnvConfigs {
            rpc_url: "https://test.com".to_string(),
            network_passphrase: "Test Network".to_string(),
        };

        let env = Env {
            configs: env_configs,
            rpc_client: Arc::new(MockRpcClient::new()),
        };

        // Test single account operations
        let account = Account::multisig(mock_signer3().account_id(), all_signers());

        // Test account loading
        let entry = account.load(&env).await;

        let expected_account_id = mock_signer3().account_id().0.to_string();
        let res_account_id = entry.unwrap().account_id.0.to_string();

        let signers = account.signers();

        for (i, sig) in signers.iter().enumerate() {
            assert_eq!(sig.account_id(), all_signers()[i].account_id())
        }

        assert_eq!(expected_account_id, res_account_id);
    }

    #[tokio::test]
    async fn sign_transaction() {
        let env_configs = EnvConfigs {
            rpc_url: "https://test.com".to_string(),
            network_passphrase: "Test Network".to_string(),
        };

        let env = Env {
            configs: env_configs,
            rpc_client: Arc::new(MockRpcClient::new()),
        };

        // Test single account operations
        let mut account = Account::single(mock_signer1());

        // Test sign transaction
        let tx = TransactionBuilder::new(&account, &env)
            .build()
            .await
            .unwrap();

        account.set_authorized_calls(1);

        let signed_tx = account.sign_transaction(&tx, &env.network_id());

        assert!(signed_tx.is_ok());
    }

    #[tokio::test]
    async fn sign_transaction_unsafe() {
        let env_configs = EnvConfigs {
            rpc_url: "https://test.com".to_string(),
            network_passphrase: "Test Network".to_string(),
        };

        let env = Env {
            configs: env_configs,
            rpc_client: Arc::new(MockRpcClient::new()),
        };

        // Test single account operations
        let mut account = Account::single(mock_signer1());

        // Test sign transaction
        let tx = TransactionBuilder::new(&account, &env)
            .build()
            .await
            .unwrap();

        // no authorized calls
        account.set_authorized_calls(0);

        // sign unsafe does not check the remaining authorized calls.
        let signed_tx = account.sign_transaction_unsafe(&tx, &env.network_id());

        assert!(signed_tx.is_ok());
    }

    #[test]
    fn test_authorized_calls_decrement() {
        let mut auth = AuthorizedCalls::new(2);
        assert!(auth.can_call());
        assert!(auth.try_decrement().is_ok());
        assert!(auth.try_decrement().is_ok());
        assert!(auth.try_decrement().is_err());
    }

    #[tokio::test]
    async fn test_configure() {
        let env_configs = EnvConfigs {
            rpc_url: "https://test.com".to_string(),
            network_passphrase: "Test Network".to_string(),
        };

        let env = Env {
            configs: env_configs,
            rpc_client: Arc::new(MockRpcClient::new()),
        };

        let account = Account::single(mock_signer1());
        let config = AccountConfig::new()
            .with_master_weight(10)
            .with_thresholds(1, 2, 3)
            .add_signer(mock_signer3().public_key(), 5);

        let tx = account.configure(&env, config).await;

        if let TransactionEnvelope::Tx(tx_env) = tx.unwrap() {
            if let OperationBody::SetOptions(op) = &tx_env.tx.operations[0].body {
                assert_eq!(
                    op.signer,
                    Some(XdrSigner {
                        key: SignerKey::Ed25519(mock_signer3().public_key().0.into()),
                        weight: 5
                    })
                );
            }

            if let OperationBody::SetOptions(op) = &tx_env.tx.operations[1].body {
                assert_eq!(op.master_weight, Some(10));
                assert_eq!(op.low_threshold, Some(1));
                assert_eq!(op.med_threshold, Some(2));
                assert_eq!(op.high_threshold, Some(3));
            }
        }
    }
}
