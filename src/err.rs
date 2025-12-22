use crate::prelude::*;

/// An error that can occur during serializing to (or deserializing from) Universal Binary JSON
#[derive(Debug)]
pub enum UbjError {
    /// Error occurring during serialization when the type of value is not supported by the Universal Binary JSON format
    UnsupportedType(&'static str),

    /// Error occurring during deserialization when a character is not valid in Universal Binary JSON format.
    CharOutOfRange(char),

    /// Any other error defined by the user of this crate
    Other(String),

    /// Error occurring IO (Input/Output) against the underlying writer/reader
    IO(IoError),
}


impl core::fmt::Display for UbjError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            UbjError::UnsupportedType(msg) =>  write!(formatter, "Unsupported type: {msg}"),
            UbjError::CharOutOfRange(c) => { let code = *c as u32; write!(formatter, "Char out of range: {code:#x}") },
            UbjError::Other(msg) =>  write!(formatter, "{msg}"),
            UbjError::IO(err) =>  write!(formatter, "IO error occurred: {}", err),
        }
    }
}


impl From<IoError> for UbjError {
    fn from(err: IoError) -> Self {
        UbjError::IO(err)
    }
}


#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::ToString;

impl serde::ser::Error for UbjError {
    fn custom<T: core::fmt::Display>(msg: T) -> Self {
        UbjError::Other(msg.to_string())
    }
}


#[cfg(feature = "std")]
impl std::error::Error for UbjError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UbjError::IO(err) => Some(err),
            _ => None
        }
    }
}

