//! # Soroban Account Guards
//! Represents a guard mechanism that can
//! be used to control and limit operations.
//!
//! ## Example
//!
//! ```rust,no_run
//! use soroban_rs::{Env, Signer, Account, Guard};
//! use ed25519_dalek::SigningKey;
//!
//! async fn example(signing_key: SigningKey) {
//!     let mut account = Account::single(Signer::new(signing_key));
//!     let guard = Guard::NumberOfAllowedCalls(3);
//!     account.add_guard(guard);
//! }
//! ```
#[derive(Clone)]
pub enum Guard {
    /// Limits the number of allowed calls to a specific operation.
    /// The u16 value represents the remaining number of calls allowed.
    NumberOfAllowedCalls(u16),
    // ... other variants
}

impl Guard {
    /// Checks if the guard condition is satisfied.
    ///
    /// # Returns
    /// * `true` if the operation is allowed to proceed
    /// * `false` if the operation should be blocked
    pub fn check(&self) -> bool {
        match self {
            Guard::NumberOfAllowedCalls(remaining) => *remaining > 0,
            // handle other variants
        }
    }

    /// Updates the guard state after an operation has been performed.
    ///
    /// This method should be called after a successful operation to update
    /// the internal state of the guard (e.g., decrement remaining allowed calls).
    pub fn update(&mut self) {
        match self {
            Guard::NumberOfAllowedCalls(remaining) => {
                if *remaining > 0 {
                    *remaining -= 1;
                }
            } // handle other variants
        }
    }
}
