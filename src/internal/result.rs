use super::error::UbjError;

/// A convenient type alias for a result able to wrap a custom UBJ error
pub type UbjResult<T> = core::result::Result<T, UbjError>;
