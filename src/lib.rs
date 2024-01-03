//! ```rust
//! # use std::io;
//! # fn app() -> anyhow::Result<()> {
//! use archiv::Compress;
//!
//! let opts = archiv::CompressOptions::default();
//! let mut archiv = opts.stream_compress(io::stdout())?;
//! archiv.write_item(b"hello")?;
//! archiv.finish()?;
//! # Ok(()) }
//! ```
//!
//! ```rust
//! # use std::io;
//! # use std::io::Read;
//! # fn app() -> anyhow::Result<()> {
//! use archiv::Expand;
//!
//! let opts = archiv::ExpandOptions::default();
//! let mut archiv = opts.stream(io::stdin().lock())?;
//! while let Some(mut item) = archiv.next_item()? {
//!     let mut s = String::new();
//!     item.read_to_string(&mut s)?;
//!     println!("{s}");
//! }
//! # Ok(()) }
//! ```
//!
mod error;
mod header;
mod read;
mod write;
mod zbuild;

pub use error::Error;
pub use read::*;
pub use write::*;

pub use zstd::dict::{DecoderDictionary, EncoderDictionary};
pub use zstd::stream::read::Decoder as ZDecoder;
