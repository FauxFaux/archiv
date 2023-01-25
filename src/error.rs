use std::backtrace::Backtrace;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("overflow during a 64-bit math operation (unlikely)")]
    LengthOverflow,
    #[error("internal IO error")]
    Io {
        #[from]
        source: io::Error,
        backtrace: Backtrace,
    },
    #[error("unexpected internal error: {0}")]
    Internal(&'static str),
}
