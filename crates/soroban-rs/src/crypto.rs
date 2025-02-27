use rand;
use sha2::{Digest, Sha256};
use stellar_xdr::curr::{
    ContractIdPreimage, ContractIdPreimageFromAddress, Hash, HashIdPreimage,
    HashIdPreimageContractId, Limits, ScAddress, Uint256, WriteXdr,
};
use crate::error::SorobanHelperError;

pub fn sha256_hash(data: &[u8]) -> Hash {
    let hash_bytes: [u8; 32] = Sha256::digest(data).into();
    Hash(hash_bytes)
}

pub fn generate_salt() -> Uint256 {
    let salt_bytes: [u8; 32] = rand::random();
    Uint256(salt_bytes)
}

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

    let preimage_xdr = preimage.to_xdr(Limits::none())
        .map_err(|e| SorobanHelperError::XdrEncodingFailed(e.to_string()))?;
    
    let contract_id = stellar_strkey::Contract(Sha256::digest(preimage_xdr).into());

    Ok(contract_id)
}