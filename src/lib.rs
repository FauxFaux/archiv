#![feature(error_generic_member_access)]
#![feature(provide_any)]

mod error;
mod write;

const HEADER: [u8; 8] = *b"arch\0\0\0\0";
const FOOTER: [u8; 8] = u64::to_ne_bytes(0xffff_fff0);
const HEADER_LEN: u64 = 8;

pub use error::Error;
pub use write::*;
