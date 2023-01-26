#![feature(error_generic_member_access)]
#![feature(provide_any)]

extern crate core;

mod error;
mod header;
mod read;
mod write;

pub use error::Error;
pub use read::*;
pub use write::*;
