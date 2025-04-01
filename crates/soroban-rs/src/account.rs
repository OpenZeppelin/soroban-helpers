//! # Soroban Account Management
//!
//! This module provides types and functionality for handling Stellar accounts in Soroban,
//! including transaction signing for both single and multi-signature (multisig) accounts.
//!
//! ## Features
//!
//! - Account sequence number tracking and management
//! - Single and multi-signature account support
//! - Transaction signing with authorization control
//! - Account configuration (thresholds, weights, signers)
//!
//! ## Example
//!
//! ```rust,no_run
//! use soroban_rs::{Account, Env, EnvConfigs, Signer};
//! use ed25519_dalek::SigningKey;
//!
//! // Example private key (32 bytes)
//! let private_key_bytes: [u8; 32] = [
//!     1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
//!     26, 27, 28, 29, 30, 31, 32,
//! ];
//!
//! // Create a signer from a secret key
//! let signing_key = SigningKey::from_bytes(&private_key_bytes);
//! let signer = Signer::new(signing_key);
//!
//! // Single-signature account
//! let account = Account::single(signer);
//! ```
use crate::{Env, Signer, TransactionBuilder, error::SorobanHelperError, guard::Guard};
use std::fmt;
use stellar_strkey::ed25519::PublicKey;
use stellar_xdr::curr::{
    AccountEntry, AccountId, DecoratedSignature, Hash, Operation, OperationBody, SetOptionsOp,
    Signer as XdrSigner, SignerKey, Transaction, TransactionEnvelope, TransactionV1Envelope, VecM,
};

/// Represents a transaction sequence number for a Stellar account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountSequence(i64);

impl AccountSequence {
    /// Creates a new sequence number with the specified value.
    ///
    /// # Parameters
    ///
    /// * `val` - The i64 sequence number value
    pub fn new(val: i64) -> Self {
        Self(val)
    }

    /// Returns a new SequenceNumber incremented by one.
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    /// Increments the sequence number and returns the new value.
    ///
    /// Unlike next() which leaves the original value unchanged, this method
    /// replaces the original sequence number.
    pub fn increment(self) -> Self {
        self.next()
    }

    /// Returns the raw i64 sequence number.
    pub fn value(self) -> i64 {
        self.0
    }
}

impl fmt::Display for AccountSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for AccountSequence {
    fn from(val: i64) -> Self {
        Self(val)
    }
}

impl From<AccountSequence> for i64 {
    fn from(seq: AccountSequence) -> Self {
        seq.0
    }
}

/// Configuration options for setting up or modifying a Stellar account.
///
/// Used to configure thresholds and signers for an account. This is particularly
/// useful for creating or modifying multisig accounts with specific
/// threshold requirements.
///
/// # Example
///
/// ```rust,no_run
/// use soroban_rs::AccountConfig;
/// use stellar_strkey::ed25519::PublicKey;
///
/// let config = AccountConfig::new()
///     .with_master_weight(10)
///     .with_thresholds(1, 5, 10)
///     .add_signer(PublicKey::from_string("PUBLIC KEY").unwrap(), 5);
/// ```
#[derive(Default, Debug, Clone)]
pub struct AccountConfig {
    /// Weight assigned to the master key (account owner)
    master_weight: Option<u32>,
    /// Threshold for low security operations
    low_threshold: Option<u32>,
    /// Threshold for medium security operations
    med_threshold: Option<u32>,
    /// Threshold for high security operations
    high_threshold: Option<u32>,
    /// Additional signers with their respective weights
    signers: Vec<(PublicKey, u32)>,
}

impl AccountConfig {
    /// Creates a new empty account configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the master key weight for the account.
    ///
    /// # Parameters
    ///
    /// * `weight` - The weight to assign to the master key
    ///   Set to 0 to prevent the master key from being used for signing
    pub fn with_master_weight(mut self, weight: u32) -> Self {
        self.master_weight = Some(weight);
        self
    }

    /// Sets the threshold values for low, medium, and high security operations.
    ///
    /// # Parameters
    ///
    /// * `low` - Threshold for low security operations (e.g., setting options)
    /// * `med` - Threshold for medium security operations (e.g., payments)
    /// * `high` - Threshold for high security operations (e.g., account merge)
    pub fn with_thresholds(mut self, low: u32, med: u32, high: u32) -> Self {
        self.low_threshold = Some(low);
        self.med_threshold = Some(med);
        self.high_threshold = Some(high);
        self
    }

    /// Adds a new signer with the specified weight.
    ///
    /// # Parameters
    ///
    /// * `key` - The public key of the signer to add
    /// * `weight` - The weight to assign to this signer
    pub fn add_signer(mut self, key: PublicKey, weight: u32) -> Self {
        self.signers.push((key, weight));
        self
    }

    /// Helper function to create a signer operation
    fn create_signer_operation(&self, public_key: &PublicKey, weight: u32) -> Operation {
        let signer_key = SignerKey::Ed25519(public_key.0.into());
        Operation {
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
        }
    }

    /// Helper function to create a thresholds operation
    fn create_thresholds_operation(&self) -> Option<Operation> {
        let has_thresholds = self.master_weight.is_some()
            || self.low_threshold.is_some()
            || self.med_threshold.is_some()
            || self.high_threshold.is_some();

        if !has_thresholds {
            return None;
        }

        Some(Operation {
            source_account: None,
            body: OperationBody::SetOptions(SetOptionsOp {
                inflation_dest: None,
                clear_flags: None,
                set_flags: None,
                master_weight: self.master_weight,
                low_threshold: self.low_threshold,
                med_threshold: self.med_threshold,
                high_threshold: self.high_threshold,
                home_domain: None,
                signer: None,
            }),
        })
    }
}

/// Represents a single-signature account.
#[derive(Clone)]
pub struct SingleAccount {
    /// The account's identifier
    account_id: AccountId,
    /// Signer associated with this account
    signer: Box<Signer>,
    /// List of guards associated with this account
    pub guards: Vec<Guard>,
}

impl SingleAccount {
    /// Creates a new single-signature account
    ///
    /// # Parameters
    ///
    /// * `signer` - The signer for this account
    /// * `authorized_calls` - The number of calls to authorize
    pub fn new(signer: Signer) -> Self {
        Self {
            account_id: signer.account_id(),
            signer: Box::new(signer),
            guards: Vec::new(),
        }
    }
}

impl fmt::Display for SingleAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SingleAccount({})", self.account_id.0)
    }
}

/// Represents a multisig account.
#[derive(Clone)]
pub struct MultisigAccount {
    /// The account's identifier
    account_id: AccountId,
    /// Signers associated with this account
    pub signers: Vec<Signer>,
    /// List of guards associated with this account
    pub guards: Vec<Guard>,
}

impl MultisigAccount {
    /// Creates a new multisig account
    ///
    /// # Parameters
    ///
    /// * `account_id` - The identifier for this account
    /// * `signers` - A vector of signers for this account
    /// * `authorized_calls` - The number of calls to authorize
    pub fn new(account_id: AccountId, signers: Vec<Signer>) -> Self {
        Self {
            account_id,
            signers,
            guards: Vec::new(),
        }
    }
}

impl fmt::Display for MultisigAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MultisigAccount({}, {} signers)",
            self.account_id.0,
            self.signers.len()
        )
    }
}

/// Represents either a single-signature or multisig account.
///
/// This is the main account type used for interacting with the Stellar network.
/// It provides methods for signing transactions, configuring account settings,
/// and managing sequence numbers.
#[derive(Clone)]
pub enum Account {
    /// Single-signature account with one key pair
    KeyPair(SingleAccount),
    /// Multi-signature account
    Multisig(MultisigAccount),
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyPair(account) => write!(f, "{}", account),
            Self::Multisig(account) => write!(f, "{}", account),
        }
    }
}

impl From<SingleAccount> for Account {
    fn from(account: SingleAccount) -> Self {
        Self::KeyPair(account)
    }
}

impl From<MultisigAccount> for Account {
    fn from(account: MultisigAccount) -> Self {
        Self::Multisig(account)
    }
}

impl From<Signer> for Account {
    fn from(signer: Signer) -> Self {
        Self::single(signer)
    }
}

// Single merged implementation block for Account
impl Account {
    // ==== Constructors ====

    /// Creates a new single-signature Account instance with the provided signer.
    ///
    /// # Parameters
    ///
    /// * `signer` - The signer for this account
    ///
    /// # Returns
    ///
    /// A new `Account` instance with the KeyPair variant
    pub fn single(signer: Signer) -> Self {
        Self::KeyPair(SingleAccount {
            signer: Box::new(signer.clone()),
            account_id: signer.account_id(),
            guards: Vec::new(),
        })
    }

    /// Creates a new multisig Account instance with the provided account ID and signers.
    ///
    /// # Parameters
    ///
    /// * `account_id` - The identifier for this account
    /// * `signers` - A vector of signers for this multi-signature account
    ///
    /// # Returns
    ///
    /// A new `Account` instance with the Multisig variant
    pub fn multisig(account_id: AccountId, signers: Vec<Signer>) -> Self {
        Self::Multisig(MultisigAccount {
            account_id,
            signers,
            guards: Vec::new(),
        })
    }

    /// Returns the account's identifier.
    pub fn account_id(&self) -> AccountId {
        match self {
            Self::KeyPair(account) => account.account_id.clone(),
            Self::Multisig(account) => account.account_id.clone(),
        }
    }

    /// Returns a reference to the account's signers.
    pub fn signers(&self) -> &[Signer] {
        match self {
            Self::KeyPair(account) => std::slice::from_ref(&*account.signer),
            Self::Multisig(account) => &account.signers,
        }
    }

    /// Loads the account information from the network.
    ///
    /// # Parameters
    /// * `env` - The environment to use for loading the account
    ///
    /// # Returns
    /// * `AccountEntry` - The account information
    pub async fn load(&self, env: &Env) -> Result<AccountEntry, SorobanHelperError> {
        env.get_account(&self.account_id().to_string()).await
    }

    /// Gets the current sequence number for the account.
    ///
    /// # Parameters
    ///
    /// * `env` - The environment to use for fetching the sequence number
    ///
    /// # Returns
    ///
    /// The current sequence number wrapped in `AccountSequence`
    pub async fn get_sequence(&self, env: &Env) -> Result<AccountSequence, SorobanHelperError> {
        let entry = self.load(env).await?;
        Ok(AccountSequence::from(entry.seq_num.0))
    }

    /// Retrieves the next available sequence number.
    ///
    /// This is useful when preparing a new transaction.
    ///
    /// # Parameters
    ///
    /// * `env` - The environment to use for fetching the sequence number
    ///
    /// # Returns
    ///
    /// The next sequence number (current + 1) wrapped in `AccountSequence`
    pub async fn next_sequence(&self, env: &Env) -> Result<AccountSequence, SorobanHelperError> {
        Ok(self.get_sequence(env).await?.next())
    }

    /// Adds a guard to the account.
    ///
    /// Guards are used to control and limit operations that can be performed with this account.
    /// Multiple guards can be added to an account, and all guards must pass for operations to proceed.
    ///
    /// # Parameters
    ///
    /// * `guard` - The guard to add to the account
    pub fn add_guard(&mut self, guard: Guard) {
        match self {
            Self::KeyPair(account) => account.guards.push(guard),
            Self::Multisig(account) => account.guards.push(guard),
        }
    }

    /// Checks if all guards associated with this account are satisfied.
    ///
    /// This method evaluates all guards attached to the account and returns
    /// true only if all of them pass their respective checks.
    ///
    /// # Returns
    ///
    /// * `false` - If any guard fails its check
    pub fn check_guards(&self, tx: &Transaction) -> Result<bool, SorobanHelperError> {
        match self {
            Self::KeyPair(account) => {
                for guard in &account.guards {
                    if !guard.check(tx)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            },
            Self::Multisig(account) => {
                for guard in &account.guards {
                    if !guard.check(tx)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            },
        }
    }

    /// Updates the state of all guards after an operation has been performed.
    ///
    /// This method should be called after a successful operation to update
    /// the internal state of all guards (e.g., decrement remaining allowed calls).
    pub fn update_guards(&mut self, tx: &Transaction) -> Result<(), SorobanHelperError> {
        match self {
            Self::KeyPair(account) => {
                for guard in &mut account.guards {
                    guard.update(tx)?;
                }
                Ok(())
            },
            Self::Multisig(account) => {
                for guard in &mut account.guards {
                    guard.update(tx)?;
                }
                Ok(())
            },
        }
    }

    /// Sign a transaction using the account's signers.
    ///
    /// # Parameters
    ///
    /// * `tx` - The transaction to sign
    /// * `network_id` - The network ID hash
    /// * `signers` - The signers to use
    ///
    /// # Returns
    ///
    /// A vector of decorated signatures
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

    /// Signs a transaction, ensuring the account still has authorized calls.
    ///
    /// Decrements the authorized call counter when successful.
    ///
    /// # Parameters
    ///
    /// * `tx` - The transaction to sign
    /// * `network_id` - The network ID hash
    ///
    /// # Returns
    ///
    /// A signed transaction envelope
    pub fn sign_transaction(
        &mut self,
        tx: &Transaction,
        network_id: &Hash,
    ) -> Result<TransactionEnvelope, SorobanHelperError> {
        if !self.check_guards(tx)? {
            return Err(SorobanHelperError::Unauthorized(
                "The transaction didn't pass one or more guards".to_string(),
            ));
        }

        let signatures = Self::sign_with_tx(tx, network_id, self.signers())?;
        self.update_guards(tx)?;

        Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: tx.clone(),
            signatures,
        }))
    }

    /// Signs a transaction envelope by appending new signatures.
    ///
    /// # Parameters
    ///
    /// * `tx_envelope` - The transaction envelope to sign
    /// * `network_id` - The network ID hash
    ///
    /// # Returns
    ///
    /// A transaction envelope with the new signatures appended
    pub fn sign_transaction_envelope(
        &mut self,
        tx_envelope: &TransactionEnvelope,
        network_id: &Hash,
    ) -> Result<TransactionEnvelope, SorobanHelperError> {
        let tx_v1 = match tx_envelope {
            TransactionEnvelope::Tx(tx_v1) => tx_v1,
            _ => {
                return Err(SorobanHelperError::XdrEncodingFailed(
                    "Invalid transaction envelope".to_string(),
                ));
            }
        };

        if !self.check_guards(&tx_v1.tx)? {
            return Err(SorobanHelperError::Unauthorized(
                "The transaction didn't pass one or more guards".to_string(),
            ));
        }

        let prev_signatures = tx_v1.signatures.clone();
        let new_signatures = Self::sign_with_signers(&tx_v1.tx, network_id, self.signers())?;

        let mut all_signatures = prev_signatures.to_vec();
        all_signatures.extend(new_signatures.to_vec());
        let signatures: VecM<DecoratedSignature, 20> = all_signatures.try_into().map_err(|_| {
            SorobanHelperError::XdrEncodingFailed(
                "Too many signatures for XDR vector (max 20)".to_string(),
            )
        })?;

        self.update_guards(&tx_v1.tx)?;

        Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: tx_v1.tx.clone(),
            signatures,
        }))
    }

    /// Configures the account by building and signing a transaction that sets options.
    ///
    /// This can be used to add signers, set thresholds, and modify other account settings.
    ///
    /// # Parameters
    ///
    /// * `env` - The environment to use for transaction building
    /// * `config` - The account configuration to apply
    ///
    /// # Returns
    ///
    /// A signed transaction envelope containing the set options operations
    pub async fn configure(
        mut self,
        env: &Env,
        config: AccountConfig,
    ) -> Result<TransactionEnvelope, SorobanHelperError> {
        let mut tx = TransactionBuilder::new(&self, env);

        // Add set options operation for each signer configuration
        for (public_key, weight) in &config.signers {
            tx = tx.add_operation(config.create_signer_operation(public_key, *weight));
        }

        // Add thresholds if any are specified
        if let Some(op) = config.create_thresholds_operation() {
            tx = tx.add_operation(op);
        }

        let tx = tx.simulate_and_build(env, &self).await?;
        self.sign_transaction(&tx, &env.network_id())
    }

    /// Signs a transaction without checking or decrementing the authorized_calls counter.
    ///
    /// This method bypasses authorization checks and should be used with caution.
    ///
    /// # Parameters
    ///
    /// * `tx` - The transaction to sign
    /// * `network_id` - The network ID hash
    ///
    /// # Returns
    ///
    /// A signed transaction envelope
    pub fn sign_transaction_unsafe(
        &self,
        tx: &Transaction,
        network_id: &Hash,
    ) -> Result<TransactionEnvelope, SorobanHelperError> {
        let signatures = Self::sign_with_signers(tx, network_id, self.signers())?;

        Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: tx.clone(),
            signatures,
        }))
    }

    /// Sign a transaction using the account's signers.
    ///
    /// # Parameters
    ///
    /// * `tx` - The transaction to sign
    /// * `network_id` - The network ID hash
    /// * `signers` - The signers to use
    ///
    /// # Returns
    ///
    /// A vector of decorated signatures
    pub fn sign_with_signers(
        tx: &Transaction,
        network_id: &Hash,
        signers: &[Signer],
    ) -> Result<VecM<DecoratedSignature, 20>, SorobanHelperError> {
        if signers.is_empty() {
            return Err(SorobanHelperError::SigningFailed(
                "No signers provided".to_string(),
            ));
        }

        let signatures: Vec<DecoratedSignature> = signers
            .iter()
            .map(|signer| signer.sign_transaction(tx, network_id))
            .collect::<Result<_, _>>()?;

        signatures.try_into().map_err(|_| {
            SorobanHelperError::XdrEncodingFailed(
                "Too many signatures for XDR vector (max 20)".to_string(),
            )
        })
    }
}/// * `true` - If all guards pass their checks (or if there are no guards)
    

#[cfg(test)]
mod test {
    use stellar_xdr::curr::{OperationBody, Signer as XdrSigner, SignerKey, TransactionEnvelope};

    use crate::account::AccountSequence;
    use crate::guard::Guard;
    use crate::mock::{all_signers, mock_env, mock_signer1, mock_signer3};
    use crate::{
        Account, AccountConfig, MultisigAccount, SingleAccount, SorobanHelperError,
        TransactionBuilder,
    };

    #[tokio::test]
    async fn load_account() {
        let env = mock_env(None, None, None);

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
        let env = mock_env(None, None, None);

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
        let env = mock_env(None, None, None);

        // Test single account operations
        let mut account = Account::single(mock_signer1());

        // Test sign transaction
        let tx = TransactionBuilder::new(&account, &env)
            .build()
            .await
            .unwrap();

        let authorized_calls = Guard::NumberOfAllowedCalls(1);
        account.add_guard(authorized_calls.clone());
        let signed_tx = account.sign_transaction(&tx, &env.network_id());

        assert!(signed_tx.is_ok());

        // when exceeded number of authorized calls
        let signed_tx_2 = account.sign_transaction(&tx, &env.network_id());
        assert!(signed_tx_2.is_err());
        // asserts it throws the right error
        assert_eq!(
            signed_tx_2.err().unwrap(),
            SorobanHelperError::Unauthorized(
                "The transaction didn't pass one or more guards".to_string()
            )
        );
    }

    #[tokio::test]
    async fn sign_transaction_unsafe() {
        let env = mock_env(None, None, None);

        // Test single account operations
        let mut account = Account::single(mock_signer1());

        // Test sign transaction
        let tx = TransactionBuilder::new(&account, &env)
            .build()
            .await
            .unwrap();

        // no authorized calls
        let authorized_calls = Guard::NumberOfAllowedCalls(0);
        account.add_guard(authorized_calls);

        // sign unsafe does not check the remaining authorized calls.
        let signed_tx = account.sign_transaction_unsafe(&tx, &env.network_id());

        assert!(signed_tx.is_ok());
    }

    #[test]
    fn test_display_account_sequence() {
        let seq = AccountSequence::new(42);
        assert_eq!(seq.to_string(), "42");
    }

    #[test]
    fn test_account_sequence_conversions() {
        // Test From<i64> for AccountSequence
        let seq_from_i64 = crate::account::AccountSequence::from(123_i64);
        assert_eq!(seq_from_i64.value(), 123);

        // Test From<AccountSequence> for i64
        let seq = crate::account::AccountSequence::new(456);
        let i64_from_seq: i64 = seq.into();
        assert_eq!(i64_from_seq, 456);
    }

    #[test]
    fn test_account_config_default() {
        let config = AccountConfig::default();

        assert_eq!(config.master_weight, None);
        assert_eq!(config.low_threshold, None);
        assert_eq!(config.med_threshold, None);
        assert_eq!(config.high_threshold, None);
        assert!(config.signers.is_empty());

        let config_new = AccountConfig::new();
        assert_eq!(config.master_weight, config_new.master_weight);
        assert_eq!(config.low_threshold, config_new.low_threshold);
        assert_eq!(config.med_threshold, config_new.med_threshold);
        assert_eq!(config.high_threshold, config_new.high_threshold);
        assert_eq!(config.signers.len(), config_new.signers.len());
    }

    #[test]
    fn test_account_display_implementations() {
        let signer = mock_signer1();
        let single_account = SingleAccount::new(signer.clone());
        let expected_single_display = format!("SingleAccount({})", signer.account_id().0);
        assert_eq!(single_account.to_string(), expected_single_display);

        let signers = all_signers();
        let account_id = mock_signer3().account_id();
        let multisig_account = MultisigAccount::new(account_id.clone(), signers.clone());
        let expected_multisig_display = format!(
            "MultisigAccount({}, {} signers)",
            account_id.0,
            signers.len()
        );
        assert_eq!(multisig_account.to_string(), expected_multisig_display);

        let account_single = Account::single(mock_signer1());
        let account_multisig = Account::multisig(mock_signer3().account_id(), all_signers());

        assert!(account_single.to_string().starts_with("SingleAccount"));
        assert!(account_multisig.to_string().starts_with("MultisigAccount"));
    }

    #[test]
    fn test_account_from_implementations() {
        let signer1 = mock_signer1();
        let single_account = SingleAccount::new(signer1);
        let expected_id = single_account.account_id.0.to_string();
        let account_from_single: Account = single_account.into();

        assert!(matches!(account_from_single, Account::KeyPair(_)));

        assert!(
            matches!(account_from_single, Account::KeyPair(inner) if inner.account_id.0.to_string() == expected_id)
        );

        let signers = all_signers();
        let account_id = mock_signer3().account_id();

        let expected_multi_id = account_id.0.to_string();
        let expected_signers_len = signers.len();

        let multisig_account = MultisigAccount::new(account_id, signers);
        let account_from_multisig: Account = multisig_account.into();

        assert!(matches!(account_from_multisig, Account::Multisig(_)));

        assert!(matches!(account_from_multisig, Account::Multisig(inner) if
            inner.account_id.0.to_string() == expected_multi_id &&
            inner.signers.len() == expected_signers_len
        ));

        let signer2 = mock_signer1();
        let expected_signer_id = signer2.account_id().0.to_string();
        let account_from_signer: Account = signer2.into();

        assert!(matches!(account_from_signer, Account::KeyPair(_)));

        assert!(
            matches!(account_from_signer, Account::KeyPair(inner) if inner.account_id.0.to_string() == expected_signer_id)
        );
    }

    #[tokio::test]
    async fn test_next_sequence() {
        let env = mock_env(None, None, None);
        let account = Account::single(mock_signer1());

        let current_seq = account.get_sequence(&env).await.unwrap();

        let next_seq = account.next_sequence(&env).await.unwrap();
        assert_eq!(next_seq.value(), current_seq.value() + 1);

        let second_current = account.get_sequence(&env).await.unwrap();
        let second_next = account.next_sequence(&env).await.unwrap();
        assert_eq!(second_current.value(), current_seq.value());

        assert_eq!(second_next.value(), second_current.value() + 1);
    }

    #[test]
    fn test_create_thresholds_operation() {
        let empty_config = AccountConfig::new();
        assert!(empty_config.create_thresholds_operation().is_none());

        let master_weight_config = AccountConfig::new().with_master_weight(5);
        assert!(master_weight_config.create_thresholds_operation().is_some());

        let thresholds_config = AccountConfig::new().with_thresholds(1, 2, 3);
        assert!(thresholds_config.create_thresholds_operation().is_some());

        let low_threshold_config = AccountConfig::new().with_thresholds(1, 0, 0);
        let low_op = low_threshold_config.create_thresholds_operation().unwrap();
        if let OperationBody::SetOptions(op_body) = &low_op.body {
            assert_eq!(op_body.low_threshold, Some(1));
            assert_eq!(op_body.med_threshold, Some(0));
            assert_eq!(op_body.high_threshold, Some(0));
        }

        let med_threshold_config = AccountConfig::new().with_thresholds(0, 2, 0);
        let med_op = med_threshold_config.create_thresholds_operation().unwrap();
        if let OperationBody::SetOptions(op_body) = &med_op.body {
            assert_eq!(op_body.low_threshold, Some(0));
            assert_eq!(op_body.med_threshold, Some(2));
            assert_eq!(op_body.high_threshold, Some(0));
        }

        let high_threshold_config = AccountConfig::new().with_thresholds(0, 0, 3);
        let high_op = high_threshold_config.create_thresholds_operation().unwrap();
        if let OperationBody::SetOptions(op_body) = &high_op.body {
            assert_eq!(op_body.low_threshold, Some(0));
            assert_eq!(op_body.med_threshold, Some(0));
            assert_eq!(op_body.high_threshold, Some(3));
        }
    }

    #[tokio::test]
    async fn test_configure() {
        let env = mock_env(None, None, None);

        let account = Account::single(mock_signer1());
        let config = AccountConfig::new()
            .with_master_weight(10)
            .with_thresholds(1, 2, 3)
            .add_signer(mock_signer3().public_key(), 5);

        let tx = account.configure(&env, config).await.unwrap();

        assert!(matches!(tx, TransactionEnvelope::Tx(_)));

        let tx_env = match tx {
            TransactionEnvelope::Tx(env) => env,
            _ => return, // Will not reach here due to previous assertion
        };

        assert_eq!(tx_env.tx.operations.len(), 2);

        assert!(matches!(
            tx_env.tx.operations[0].body,
            OperationBody::SetOptions(_)
        ));

        if let OperationBody::SetOptions(op0_body) = &tx_env.tx.operations[0].body {
            assert_eq!(
                op0_body.signer,
                Some(XdrSigner {
                    key: SignerKey::Ed25519(mock_signer3().public_key().0.into()),
                    weight: 5
                })
            );
        }

        assert!(matches!(
            tx_env.tx.operations[1].body,
            OperationBody::SetOptions(_)
        ));

        if let OperationBody::SetOptions(op1_body) = &tx_env.tx.operations[1].body {
            assert_eq!(op1_body.master_weight, Some(10));
            assert_eq!(op1_body.low_threshold, Some(1));
            assert_eq!(op1_body.med_threshold, Some(2));
            assert_eq!(op1_body.high_threshold, Some(3));
        }
    }

    #[tokio::test]
    async fn test_sign_transaction_envelope() {
        let env = mock_env(None, None, None);

        let mut first_account = Account::single(mock_signer1());
        let mut second_account = Account::single(mock_signer3());

        let authorized_calls = Guard::NumberOfAllowedCalls(1);
        first_account.add_guard(authorized_calls.clone());
        second_account.add_guard(authorized_calls);

        let tx = TransactionBuilder::new(&first_account, &env)
            .build()
            .await
            .unwrap();

        let first_signed_envelope = first_account
            .sign_transaction(&tx, &env.network_id())
            .unwrap();

        assert!(matches!(first_signed_envelope, TransactionEnvelope::Tx(_)));

        let first_envelope_signatures = match &first_signed_envelope {
            TransactionEnvelope::Tx(tx_v1) => &tx_v1.signatures,
            _ => &[][..], // Empty slice as fallback, assertion below will fail if this happens
        };
        assert_eq!(first_envelope_signatures.len(), 1); // First envelope should have exactly one signature

        let final_envelope = second_account
            .sign_transaction_envelope(&first_signed_envelope, &env.network_id())
            .unwrap();

        assert!(matches!(final_envelope, TransactionEnvelope::Tx(_)));

        let final_signatures = match &final_envelope {
            TransactionEnvelope::Tx(tx_v1) => &tx_v1.signatures,
            _ => &[][..], // Empty slice as fallback, assertion below will fail if this happens
        };
        assert_eq!(final_signatures.len(), 2); // Final envelope should have exactly two signatures

        let first_public_key = mock_signer1().public_key();
        let second_public_key = mock_signer3().public_key();
        assert_eq!(final_signatures[0].hint.0, &first_public_key.0[28..32]); // First signature should match first account's public key
        assert_eq!(final_signatures[1].hint.0, &second_public_key.0[28..32]); // Second signature should match second account's public key
    }
}
