pub use anyhow::{Context, Result as AnyResult};
use nom::{error::{ErrorKind, FromExternalError, ParseError}};
use thiserror;

#[macro_export]
macro_rules! elog {
    ($msg:literal $(,)?) => {
        anyhow::anyhow!(format!("[{}].[{}]: {}", file!(), line!(), $msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        anyhow::anyhow!(format!("[{}].[{}]: {}", file!(), line!(), format!($fmt, $($arg)*)))
    };
}

#[derive(Debug, thiserror::Error)]
pub enum WikitError {
    #[error("plain error: {0}")]
    Plain(String),

    #[error("oops, error happens: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("IO error")]
    IOError(#[from] std::io::Error),

    #[error("TOML error")]
    TOMLError(#[from] toml::de::Error),

    #[error("FST error")]
    FSTError(#[from] fst::Error),

    #[error("UTF8 error")]
    UTF8Error(#[from] std::string::FromUtf8Error),

    #[error("FSTLevenshteinError error")]
    FSTLevenshteinError(#[from] fst::automaton::LevenshteinError),

    #[error("Reqwest Error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("json5 error")]
    JSON5Error(#[from] json5::Error),

    #[error("{0}")]
    SQLiteError(#[from] rusqlite::Error),
}

// TODO(2022-04-04): refactor out WikitResult<T> and use Result<T>
pub type WikitResult<T> = std::result::Result<T, WikitError>;
pub type NomResult<'a, O> = std::result::Result<(&'a [u8], O), nom::Err<WikitError>>;
pub type Result<T> = std::result::Result<T, WikitError>;

impl WikitError {
    pub fn new<S>(msg: S) -> Self where S: AsRef<str> {
        WikitError::Plain(msg.as_ref().to_string())
    }
}

impl<I> ParseError<I> for WikitError {
  fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
    WikitError::Anyhow(anyhow::anyhow!(format!("{}", kind.description())))
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

impl<I> FromExternalError<I, anyhow::Error> for WikitError {
    fn from_external_error(_input: I, _kind: ErrorKind, e: anyhow::Error) -> Self {
        WikitError::Anyhow(e)
    }
}
// Convert WikitError into string which is useful for passing error from rust into js
impl From<WikitError> for String {
    fn from(e: WikitError) -> Self {
        format!("{:?}", e)
    }
}
