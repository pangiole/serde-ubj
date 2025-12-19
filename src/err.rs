
/// An error that can occur during serializing to (or deserializing from) Universal Binary JSON
#[derive(Debug)]
pub enum UbjError {
    /// Error occurring during serialization when the type of value is not supported by the Universal Binary JSON format
    UnsupportedType(&'static str),
    /// Error occurring during deserialization when a character is not valid in Universal Binary JSON format.
    OutOfRange(char),
    /// Error occurring IO (Input/Output) against the underlying writer/reader
    IO(std::io::Error),
    /// Any other error defined by the user of this crate
    Other(String)
}


impl std::fmt::Display for UbjError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UbjError::UnsupportedType(msg) =>  write!(formatter, "Unsupported type: {msg}"),
            UbjError::OutOfRange(c) =>  write!(formatter, "Out of 0..127 range: {c}"),
            UbjError::IO(err) =>  write!(formatter, "IO error occurred: {}", err),
            UbjError::Other(msg) =>  write!(formatter, "{msg}")
        }
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

#[cfg(feature = "std")]
impl From<std::io::Error> for UbjError {
    fn from(err: std::io::Error) -> Self {
        UbjError::IO(err)
    }
}


impl serde::ser::Error for UbjError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        UbjError::Other(msg.to_string())
    }
}


#[cfg(test)]
mod tests {
    use std::error::Error;
    use super::*;
    use std::io;

    #[test]
    fn display_message_error() {
        let err = UbjError::Other(String::from("An error occurred"));
        assert_eq!(err.to_string().as_str(), "An error occurred");
    }

    #[test]
    fn display_io_error() {
        let io_err = io::Error::new(io::ErrorKind::Other, "Disk failure");
        let err = UbjError::IO(io_err);
        assert_eq!(err.to_string().as_str(), "IO error occurred: Disk failure");
    }


    #[test]
    fn source_message_error() {
        let err = UbjError::Other(String::from("An error occurred"));
        assert!(err.source().is_none());
    }

    #[test]
    fn source_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let err = UbjError::IO(io_err);
        assert!(err.source().is_some());
        assert_eq!(err.source().unwrap().to_string().as_str(), "File not found");
    }

    // TODO How can we test custom() function associated to serde::ser::Error type?
}