use stellar_xdr::curr::{AccountId, ScAddress, ScString, ScVal, StringM};
use crate::SorobanHelperError;

/// A trait for converting a value into a `ScVal`.
pub trait IntoScVal {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError>;
    fn into_val(&self) -> ScVal;
}

impl IntoScVal for AccountId {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::Address(ScAddress::Account(self.clone())))
    }

    fn into_val(&self) -> ScVal {
        ScVal::Address(ScAddress::Account(self.clone()))
    }
}

impl IntoScVal for u32 {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::U32(*self))
    }

    fn into_val(&self) -> ScVal {
        ScVal::U32(*self)
    }
}

impl IntoScVal for u64 {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::U64(*self))
    }

    fn into_val(&self) -> ScVal {
        ScVal::U64(*self)
    }
}

impl IntoScVal for i32 {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::I32(*self))
    }

    fn into_val(&self) -> ScVal {
        ScVal::I32(*self)
    }
}

impl IntoScVal for i64 {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::I64(*self))
    }

    fn into_val(&self) -> ScVal {
        ScVal::I64(*self)
    }
}

impl IntoScVal for bool {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        Ok(ScVal::Bool(*self))
    }

    fn into_val(&self) -> ScVal {
        ScVal::Bool(*self)
    }
}

impl IntoScVal for String {
    fn try_into_val(&self) -> Result<ScVal, SorobanHelperError> {
        let string_m = StringM::<{ u32::MAX }>::try_from(self)
            .map_err(|_| SorobanHelperError::XdrEncodingFailed("Failed to convert String to StringM".to_string()))?;
        Ok(ScVal::String(ScString::from(string_m)))
    }

    fn into_val(&self) -> ScVal {
        let string_m = StringM::<{ u32::MAX }>::try_from(self)
            .expect("Failed to convert String to StringM");
        ScVal::String(ScString::from(string_m))
    }
}