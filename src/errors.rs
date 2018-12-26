use std::error;
use std::fmt;

/// An error that can occur in this crate.
///
/// Generally, this error corresponds to problems parsing the input, or
/// asking for incompatible actions, or asking an unreasonable amount or rolls
#[derive(Clone, Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }

    pub(crate) fn parse<E: error::Error>(err: E) -> Error {
        Error {
            kind: ErrorKind::Parse(err.to_string()),
        }
    }

    /// Return the kind of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Error::parse(error)
    }
}

/// The kind of an error that can occur.
#[derive(Clone, Debug)]
pub enum ErrorKind {
    /// An error that occurred as a result of parsing a roll request.
    /// This can be a syntax error.
    ///
    /// The string here is the underlying error converted to a string.
    Parse(String),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::Parse(_) => "parse error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Parse(ref s) => write!(f, "parse error: {}", s),
        }
    }
}
