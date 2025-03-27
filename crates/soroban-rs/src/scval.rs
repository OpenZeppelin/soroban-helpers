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

impl IntoScVal for AccountId {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::Address(ScAddress::Account(self.clone())))
    }

    fn into_val(self) -> ScVal {
        ScVal::Address(ScAddress::Account(self))
    }
}

impl IntoScVal for u32 {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::U32(*self))
    }

    fn into_val(self) -> ScVal {
        ScVal::U32(self)
    }
}

impl IntoScVal for u64 {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::U64(*self))
    }

    fn into_val(self) -> ScVal {
        ScVal::U64(self)
    }
}

impl IntoScVal for i32 {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::I32(*self))
    }

    fn into_val(self) -> ScVal {
        ScVal::I32(self)
    }
}

impl IntoScVal for i64 {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::I64(*self))
    }

    fn into_val(self) -> ScVal {
        ScVal::I64(self)
    }
}

impl IntoScVal for bool {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::Bool(*self))
    }

    fn into_val(self) -> ScVal {
        ScVal::Bool(self)
    }
}

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
