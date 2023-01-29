use std::collections::TryReserveError;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

/// Library specific errors we can encounter, typically around underlying or format errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("this isn't the right type of file for us")]
    MagicMissing,
    #[error("this looks like the right kind of file, but uses features we can't handle")]
    MagicUnrecognised,

    #[error("an item exceeded the specified limits")]
    InvalidItem,
    #[error("invalid use of the API")]
    ApiMisuse,

    #[error("overflow during a 64-bit math operation (unlikely)")]
    LengthOverflow,
    #[error("underlying IO error")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("underlying allocator error")]
    TryReserve {
        #[from]
        source: TryReserveError,
    },
    #[error("unexpected internal error: {0}")]
    Internal(&'static str),
}
