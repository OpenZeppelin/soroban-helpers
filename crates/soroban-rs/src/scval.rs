use std::time::Duration;

use crate::SorobanHelperError;
use stellar_xdr::curr::{
    AccountId, BytesM, Duration as XDRDuration, ScAddress, ScBytes, ScString, ScVal, ScVec,
    StringM, VecM,
};

/// A trait for converting native rust values into a `ScVal`.
pub trait IntoScVal {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError>;
    fn into_val(self) -> ScVal;
}

/// Converts a Stellar `AccountId` into an `ScVal::Address` containing an account.
impl IntoScVal for AccountId {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::Address(ScAddress::Account(self.clone())))
    }

    fn into_val(self) -> ScVal {
        ScVal::Address(ScAddress::Account(self))
    }
}

/// Converts a 32-bit unsigned integer into an `ScVal::U32`.
impl IntoScVal for u32 {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::U32(*self))
    }

    fn into_val(self) -> ScVal {
        ScVal::U32(self)
    }
}

/// Converts a 64-bit unsigned integer into an `ScVal::U64`.
impl IntoScVal for u64 {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::U64(*self))
    }

    fn into_val(self) -> ScVal {
        ScVal::U64(self)
    }
}

/// Converts a 32-bit signed integer into an `ScVal::I32`.
impl IntoScVal for i32 {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::I32(*self))
    }

    fn into_val(self) -> ScVal {
        ScVal::I32(self)
    }
}

/// Converts a 64-bit signed integer into an `ScVal::I64`.
impl IntoScVal for i64 {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::I64(*self))
    }

    fn into_val(self) -> ScVal {
        ScVal::I64(self)
    }
}

/// Converts a boolean value into an `ScVal::Bool`.
impl IntoScVal for bool {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::Bool(*self))
    }

    fn into_val(self) -> ScVal {
        ScVal::Bool(self)
    }
}

/// Converts a Rust `String` into an `ScVal::String` by first converting to a Stellar `StringM`.
impl IntoScVal for String {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        let string_m = StringM::<{ u32::MAX }>::try_from(self).map_err(|_| {
            SorobanHelperError::XdrEncodingFailed("Failed to convert String to StringM".to_string())
        })?;
        Ok(ScVal::String(ScString::from(string_m)))
    }

    fn into_val(self) -> ScVal {
        let string_m =
            StringM::<{ u32::MAX }>::try_from(self).expect("Failed to convert String to StringM");
        ScVal::String(ScString::from(string_m))
    }
}

/// Converts a 32-byte array into an `ScVal::Bytes`.
impl IntoScVal for [u8; 32] {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        let bytes_m = BytesM::<{ u32::MAX }>::try_from(self).map_err(|_| {
            SorobanHelperError::XdrEncodingFailed("Failed to convert Bytes to BytesM".to_string())
        })?;
        Ok(ScVal::Bytes(ScBytes::from(bytes_m)))
    }

    fn into_val(self) -> ScVal {
        let bytes_m =
            BytesM::<{ u32::MAX }>::try_from(self).expect("Failed to convert Bytes to BytesM");
        ScVal::Bytes(ScBytes::from(bytes_m))
    }
}

/// Converts a Rust `Duration` into an `ScVal::Duration` using seconds as the time unit.
impl IntoScVal for Duration {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        let milis: u64 = self.as_secs();
        Ok(ScVal::Duration(XDRDuration::from(milis)))
    }

    fn into_val(self) -> ScVal {
        let milis: u64 = self.as_secs();
        ScVal::Duration(XDRDuration::from(milis))
    }
}

/// Converts a vector of `ScVal` objects into an `ScVal::Vec`.
impl IntoScVal for Vec<ScVal> {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        let vec_m = VecM::try_from(self).map_err(|_| {
            SorobanHelperError::XdrEncodingFailed("Failed to convert Vec to VecM".to_string())
        })?;
        Ok(ScVal::Vec(Some(ScVec::from(vec_m))))
    }

    fn into_val(self) -> ScVal {
        let vec_m = VecM::try_from(self).expect("Failed to convert Vec to VecM");
        ScVal::Vec(Some(ScVec::from(vec_m)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_xdr::curr::{PublicKey, Uint256};

    #[test]
    fn test_account_id_into_scval() {
        let public_key = PublicKey::PublicKeyTypeEd25519(Uint256([0; 32]));
        let account_id = AccountId(public_key);

        // Test try_into_val
        let scval = account_id.try_into_val().unwrap();
        match scval {
            ScVal::Address(ScAddress::Account(id)) => {
                assert_eq!(id, account_id);
            }
            _ => panic!(
                "Expected ScVal::Address(ScAddress::Account), got {:?}",
                scval
            ),
        }

        // Test into_val
        let scval = account_id.clone().into_val();
        match scval {
            ScVal::Address(ScAddress::Account(id)) => {
                assert_eq!(id, account_id);
            }
            _ => panic!(
                "Expected ScVal::Address(ScAddress::Account), got {:?}",
                scval
            ),
        }
    }

    #[test]
    fn test_u32_into_scval() {
        let value: u32 = 42;

        // Test try_into_val
        let scval = value.try_into_val().unwrap();
        match scval {
            ScVal::U32(val) => {
                assert_eq!(val, value);
            }
            _ => panic!("Expected ScVal::U32, got {:?}", scval),
        }

        // Test into_val
        let scval = value.into_val();
        match scval {
            ScVal::U32(val) => {
                assert_eq!(val, value);
            }
            _ => panic!("Expected ScVal::U32, got {:?}", scval),
        }
    }

    #[test]
    fn test_u64_into_scval() {
        let value: u64 = 42;

        // Test try_into_val
        let scval = value.try_into_val().unwrap();
        match scval {
            ScVal::U64(val) => {
                assert_eq!(val, value);
            }
            _ => panic!("Expected ScVal::U64, got {:?}", scval),
        }

        // Test into_val
        let scval = value.into_val();
        match scval {
            ScVal::U64(val) => {
                assert_eq!(val, value);
            }
            _ => panic!("Expected ScVal::U64, got {:?}", scval),
        }
    }

    #[test]
    fn test_i32_into_scval() {
        let value: i32 = -42;

        // Test try_into_val
        let scval = value.try_into_val().unwrap();
        match scval {
            ScVal::I32(val) => {
                assert_eq!(val, value);
            }
            _ => panic!("Expected ScVal::I32, got {:?}", scval),
        }

        // Test into_val
        let scval = value.into_val();
        match scval {
            ScVal::I32(val) => {
                assert_eq!(val, value);
            }
            _ => panic!("Expected ScVal::I32, got {:?}", scval),
        }
    }

    #[test]
    fn test_i64_into_scval() {
        let value: i64 = -42;

        // Test try_into_val
        let scval = value.try_into_val().unwrap();
        match scval {
            ScVal::I64(val) => {
                assert_eq!(val, value);
            }
            _ => panic!("Expected ScVal::I64, got {:?}", scval),
        }

        // Test into_val
        let scval = value.into_val();
        match scval {
            ScVal::I64(val) => {
                assert_eq!(val, value);
            }
            _ => panic!("Expected ScVal::I64, got {:?}", scval),
        }
    }

    #[test]
    fn test_bool_into_scval() {
        // Test true value
        let value = true;

        let scval = value.try_into_val().unwrap();
        match scval {
            ScVal::Bool(val) => {
                assert_eq!(val, value);
            }
            _ => panic!("Expected ScVal::Bool, got {:?}", scval),
        }

        let scval = value.into_val();
        match scval {
            ScVal::Bool(val) => {
                assert_eq!(val, value);
            }
            _ => panic!("Expected ScVal::Bool, got {:?}", scval),
        }

        // Test false value
        let value = false;

        let scval = value.try_into_val().unwrap();
        match scval {
            ScVal::Bool(val) => {
                assert_eq!(val, value);
            }
            _ => panic!("Expected ScVal::Bool, got {:?}", scval),
        }
    }

    #[test]
    fn test_string_into_scval() {
        let value = "test string".to_string();

        // Test try_into_val
        let scval = value.try_into_val().unwrap();
        match scval {
            ScVal::String(sc_string) => {
                let string_value: String = sc_string.to_utf8_string_lossy();
                assert_eq!(string_value, "test string");
            }
            _ => panic!("Expected ScVal::String, got {:?}", scval),
        }

        // Test into_val
        let value = "test string".to_string();
        let scval = value.into_val();
        match scval {
            ScVal::String(sc_string) => {
                let string_value: String = sc_string.to_utf8_string_lossy();
                assert_eq!(string_value, "test string");
            }
            _ => panic!("Expected ScVal::String, got {:?}", scval),
        }
    }

    #[test]
    fn test_bytes_into_scval() {
        let value = [42u8; 32];

        // Test try_into_val
        let scval = value.try_into_val().unwrap();
        match scval {
            ScVal::Bytes(sc_bytes) => {
                assert_eq!(sc_bytes.as_slice(), &value);
            }
            _ => panic!("Expected ScVal::Bytes, got {:?}", scval),
        }

        // Test into_val
        let scval = value.into_val();
        match scval {
            ScVal::Bytes(sc_bytes) => {
                assert_eq!(sc_bytes.as_slice(), &value);
            }
            _ => panic!("Expected ScVal::Bytes, got {:?}", scval),
        }
    }

    #[test]
    fn test_duration_into_scval() {
        let value = Duration::from_secs(42);

        // Test try_into_val
        let scval = value.try_into_val().unwrap();
        match scval {
            ScVal::Duration(xdr_duration) => {
                assert_eq!(xdr_duration.0, 42);
            }
            _ => panic!("Expected ScVal::Duration, got {:?}", scval),
        }

        // Test into_val
        let scval = value.into_val();
        match scval {
            ScVal::Duration(xdr_duration) => {
                assert_eq!(xdr_duration.0, 42);
            }
            _ => panic!("Expected ScVal::Duration, got {:?}", scval),
        }
    }

    #[test]
    fn test_vec_scval_into_scval() {
        let values = vec![ScVal::U32(1), ScVal::I32(-1), ScVal::Bool(true)];

        // Test try_into_val
        let scval = values.try_into_val().unwrap();
        match scval {
            ScVal::Vec(Some(sc_vec)) => {
                let vec_values: Vec<ScVal> = sc_vec.0.to_vec();
                assert_eq!(vec_values.len(), 3);
                assert_eq!(vec_values[0], ScVal::U32(1));
                assert_eq!(vec_values[1], ScVal::I32(-1));
                assert_eq!(vec_values[2], ScVal::Bool(true));
            }
            _ => panic!("Expected ScVal::Vec, got {:?}", scval),
        }

        // Test into_val
        let values = vec![ScVal::U32(1), ScVal::I32(-1), ScVal::Bool(true)];
        let scval = values.into_val();
        match scval {
            ScVal::Vec(Some(sc_vec)) => {
                let vec_values: Vec<ScVal> = sc_vec.0.to_vec();
                assert_eq!(vec_values.len(), 3);
                assert_eq!(vec_values[0], ScVal::U32(1));
                assert_eq!(vec_values[1], ScVal::I32(-1));
                assert_eq!(vec_values[2], ScVal::Bool(true));
            }
            _ => panic!("Expected ScVal::Vec, got {:?}", scval),
        }
    }

    #[test]
    fn test_string_conversion_error() {
        let result = StringM::<{ u32::MAX }>::try_from("test".to_string());
        assert!(result.is_ok(), "Small string should convert successfully");
    }

    #[test]
    fn test_vec_conversion_error() {
        let small_vec = vec![ScVal::U32(1), ScVal::U32(2)];
        let result: Result<VecM<ScVal, { u32::MAX }>, _> = VecM::try_from(&small_vec);
        assert!(result.is_ok(), "Small vector should convert successfully");
    }
}
