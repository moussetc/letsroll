use std::error;
use std::fmt;

use pest;

/// An error that can occur in this crate.
///
/// Generally, this error corresponds to problems parsing the input, or
/// asking for incompatible actions, or asking an unreasonable amount or rolls
#[derive(Clone, Debug, PartialEq)]
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

    pub(crate) fn file<E: error::Error>(err: E) -> Error {
        Error {
            kind: ErrorKind::File(err.to_string()),
        }
    }

    pub(crate) fn incompatible(action: &String, roll_type: &String) -> Error {
        Error {
            kind: ErrorKind::IncompatibleAction(format!(
                "Action {:?} not supported by roll type {:?}",
                action, roll_type
            )),
        }
    }

    pub(crate) fn bad_action_parameter(message: &String) -> Error {
        Error {
            kind: ErrorKind::BadActionParameter(message.clone()),
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

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::file(error)
    }
}

impl<R: pest::RuleType> From<pest::error::Error<R>> for Error {
    fn from(error: pest::error::Error<R>) -> Self {
        Error::parse(error)
    }
}

/// The kind of an error that can occur.
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    /// An error that occurred as a result of parsing a request.
    /// This can be a syntax error.
    ///
    /// The string here is the underlying error converted to a string.
    Parse(String),

    /// An error that occurred as a result of parsing a dice request.
    ///
    /// The string here is a detailed explanation of what caused the parsing to fail.
    ParseDice(String),

    /// An error that occurred as a result of trying to apply an action to an incompatible type of rolls.
    IncompatibleAction(String),

    // Occurs when dice initialization fails because of bad parameters
    BadDice(String),

    // Occurs when file operations fail
    File(String),

    // Occurs when an action parameter is invalid
    BadActionParameter(String),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::Parse(_) => "Request parsing error",
            ErrorKind::ParseDice(_) => "Dice parsing error",
            ErrorKind::IncompatibleAction(_) => "Action applying error",
            ErrorKind::BadDice(_) => "Dice creation error",
            ErrorKind::File(_) => "File operation error",
            ErrorKind::BadActionParameter(_) => "Bad action parameter error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Parse(ref s) => write!(f, "Request parse error: {}", s),
            ErrorKind::ParseDice(ref s) => write!(f, "Dice parsing error: {}", s),
            ErrorKind::IncompatibleAction(ref s) => write!(f, "Action applying error: {}", s),
            ErrorKind::BadDice(ref s) => write!(f, "Dice creation error: {}", s),
            ErrorKind::File(ref s) => write!(f, "File operation error: {}", s),
            ErrorKind::BadActionParameter(ref s) => write!(f, "Bad action parameter error {}", s),
        }
    }
}
