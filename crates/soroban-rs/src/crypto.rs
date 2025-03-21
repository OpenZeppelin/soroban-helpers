//! # Soroban Cryptography Utilities
//!
//! This module provides cryptographic functions for Soroban contracts, including
//! hashing, random salt generation, and contract ID calculation.
use crate::error::SorobanHelperError;
use sha2::{Digest, Sha256};
use stellar_xdr::curr::{
    ContractIdPreimage, ContractIdPreimageFromAddress, Hash, HashIdPreimage,
    HashIdPreimageContractId, Limits, ScAddress, Uint256, WriteXdr,
};

/// Computes the SHA-256 hash of the provided data.
///
/// # Parameters
///
/// * `data` - The byte slice to hash
///
/// # Returns
///
/// The SHA-256 hash as a `Hash` type
pub fn sha256_hash(data: &[u8]) -> Hash {
    let hash_bytes: [u8; 32] = Sha256::digest(data).into();
    Hash(hash_bytes)
}

/// Generates a random salt.
///
/// # Returns
///
/// A random 32-byte salt as `Uint256`
pub fn generate_salt() -> Uint256 {
    let salt_bytes: [u8; 32] = rand::random();
    Uint256(salt_bytes)
}

/// Calculates a contract ID from an account ID, salt, and network ID.
///
/// # Parameters
///
/// * `account_id` - The account ID of the deployer
/// * `salt` - A random salt to ensure uniqueness
/// * `network_id` - The network ID hash
///
/// # Returns
///
/// The calculated contract ID or an error if XDR encoding fails
///
/// # Errors
///
/// Returns `SorobanHelperError::XdrEncodingFailed` if the HashIdPreimage
/// cannot be encoded to XDR format
pub fn calculate_contract_id(
    account_id: &stellar_xdr::curr::AccountId,
    salt: &Uint256,
    network_id: &Hash,
) -> Result<stellar_strkey::Contract, SorobanHelperError> {
    let contract_id_preimage = ContractIdPreimage::Address(ContractIdPreimageFromAddress {
        address: ScAddress::Account(account_id.clone()),
        salt: salt.clone(),
    });

    let preimage = HashIdPreimage::ContractId(HashIdPreimageContractId {
        network_id: network_id.clone(),
        contract_id_preimage,
    });

    let preimage_xdr = preimage
        .to_xdr(Limits::none())
        .map_err(|e| SorobanHelperError::XdrEncodingFailed(e.to_string()))?;

    let contract_id = stellar_strkey::Contract(Sha256::digest(preimage_xdr).into());

    Ok(contract_id)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash() {
        let data = b"test data";
        let hash = sha256_hash(data);
        let expected_hash = "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9";

        assert_eq!(
            hash.to_string(),
            expected_hash,
            "Hash value should match the expected SHA256 hash"
        );
    }

    #[test]
    fn test_generate_salt_chi_squared() {
        // Testing Random Number Generators
        // Chai-square test for uniformity
        // https://www.cs.rice.edu/~johnmc/comp528/lecture-notes/Lecture22.pdf

        // Generate a large number of salts
        let num_salts = 10000;
        let bytes_per_salt = 32;

        // Count occurrences of each byte value (0-255)
        let mut observed = [0; 256];

        for _ in 0..num_salts {
            let salt = generate_salt();
            for &byte in salt.as_slice() {
                observed[byte as usize] += 1;
            }
        }

        // Calculate expected count for uniform distribution
        let expected = (num_salts * bytes_per_salt) as f64 / 256.0;

        // Calculate chi-squared statistic
        // χ^2 = sum[(observed - expected)^2 / expected]
        let chi_squared: f64 = observed
            .iter()
            .map(|&count| {
                let diff = count as f64 - expected;
                (diff * diff) / expected
            })
            .sum();

        // Two sided Critical value for chi-squared with df=255 and α=0.001 for each tail
        // Degrees of freedom is (number of categories - 1) = 255
        // Alpha of 0.001 is the 0.1% significance level
        // This is a conservative threshold that indicates the distribution
        // is likely not uniform if exceeded
        // Approximate values for df=255, α=0.001 (Inverse CDF of ChiSquared distribution)
        let upper_critical_value = 330.52;
        let lower_critical_value = 190.87;

        // Assert that our chi-squared value doesn't exceed or fall below the critical values
        // If it does, it suggests the distribution is not uniform
        assert!(
            chi_squared > lower_critical_value && chi_squared < upper_critical_value,
            "Chi-squared test failed: Chi-squared value ({}) outside acceptable range ({} to {})",
            chi_squared,
            lower_critical_value,
            upper_critical_value
        );
    }

    #[test]
    fn test_calculate_contract_id() {
        let public_key = stellar_xdr::curr::PublicKey::PublicKeyTypeEd25519([0; 32].into());
        let account_id = stellar_xdr::curr::AccountId(public_key);
        let salt = generate_salt();
        let network_id = Hash([0; 32]);

        match calculate_contract_id(&account_id, &salt, &network_id) {
            Ok(contract_id) => {
                assert_eq!(contract_id.0.len(), 32);
                assert!(
                    contract_id.0.iter().any(|&x| x != 0),
                    "Contract ID should not be all zeros"
                );
            }
            Err(e) => panic!("Failed to calculate contract id: {}", e),
        }
    }
}
