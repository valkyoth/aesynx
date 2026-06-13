use aesynx_abi::ObjectId;
use core::fmt;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModelObjectId(ObjectId);

impl ModelObjectId {
    pub const fn new(value: u128) -> Result<Self, ModelObjectIdError> {
        if value == 0 {
            return Err(ModelObjectIdError::Zero);
        }
        Ok(Self(ObjectId::new(value)))
    }

    pub const fn from_object_id(value: ObjectId) -> Result<Self, ModelObjectIdError> {
        if value.get() == 0 {
            return Err(ModelObjectIdError::Zero);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> ObjectId {
        self.0
    }

    pub const fn raw(self) -> u128 {
        self.0.get()
    }
}

impl fmt::Debug for ModelObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ModelObjectId")
            .field(&format_args!("<redacted>"))
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelObjectIdError {
    Zero,
}
