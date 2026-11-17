use core::fmt;
use thiserror::Error;

/// An error encountered while serializing to (or deserializing from) Universal Binary JSON
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum UbjError {

    /// Invalid byte marker was encountered
    #[error("Illegal marker: {0:#X}")]
    IllegalMarker(u8),

    /// Unsupported
    #[error("Unsupported: {0}")]
    Unsupported(&'static str),

    /// Illegal key type
    #[error("Illegal key type: {0}")]
    IllegalKeyType(&'static str),

    /// Non ASCII character was encountered
    #[error("Char not within ASCII range: {0:#X}")]
    NotAsciiChar(u32),

    /// Unexpected end of file
    #[error("Unexpected end of file")]
    UnexpectedEof,

    /// Bad String caused by UTF-8 encoding error
    #[error("Bad String: {0}")]
    BadString(#[from] core::str::Utf8Error),

    /// Error if the buffer is too small
    #[error("Buffer too small. Consider increasing its capacity to at least {0}")]
    BufferTooSmall(usize),

    /// An enum variant index is larger than the maximum value allowed by the format.
    #[error("Enum variant index is too large: {0}")]
    EnumVariantIndexTooLarge(u32),

    /// IO error caused by embedded read/write operations
    #[error("I/O error occurred: {0}")]
    IO(#[from] embedded_io::ErrorKind),

    /// Custom error  as defined by the user of this crate
    #[cfg(feature = "alloc")]
    #[error("{0}")]
    Custom(alloc::string::String),

    /// Custom error  as defined by the user of this crate
    #[cfg(not(feature = "alloc"))]
    #[error("")]
    Custom,
}

impl serde::ser::Error for UbjError {
    fn custom<T: fmt::Display>(msg: T) -> UbjError {
        #[cfg(feature = "alloc")] {
            UbjError::Custom(alloc::string::ToString::to_string(&msg))
        }
        #[cfg(not(feature = "alloc"))] {
            let _ = msg;
            UbjError::Custom
        }
    }
}


impl serde::de::Error for UbjError {
    fn custom<T: fmt::Display>(msg: T) -> UbjError {
        #[cfg(feature = "alloc")] {
            UbjError::Custom(alloc::string::ToString::to_string(&msg))
        }
        #[cfg(not(feature = "alloc"))] {
            let _ = msg;
            UbjError::Custom
        }
    }
}


#[cfg(test)]
mod test {
    use super::UbjError;

    #[cfg(feature = "alloc")]
    #[test]
    fn should_display_ubj_error_not_ascii_char() {
        let e = UbjError::NotAsciiChar(0xFF);
        assert_eq!(alloc::string::ToString::to_string(&e), "Char not within ASCII range: 0xFF");
    }

    // TODO Add more unit test cases here …
}
