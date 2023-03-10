use std::io;
use std::io::{BufRead, Read};
use zstd::dict::DecoderDictionary;

use crate::error::{Error, Result};
use crate::header::{parse_header, Kinds, HEADER_TEMPLATE, ZSTD_MAGIC, MAX_ITEM_SIZE};
use crate::zbuild::DecoderDict;

/// Entry point for expansion (reading)
pub struct ExpandOptions<'d> {
    max_item_size: u64,
    zstd: DecoderDict<'d>,
}

impl Default for ExpandOptions<'static> {
    fn default() -> Self {
        const GIGABYTE: u64 = 1024 * 1024 * 1024;
        ExpandOptions {
            max_item_size: 2 * GIGABYTE,
            zstd: DecoderDict::default(),
        }
    }
}

/// Trait for reading from compressed streams
pub trait Expand {
    fn next_item(&mut self) -> Result<Option<Box<dyn Read + '_>>>;
}

/// Concrete implementation of the compressed stream reader
pub struct ExpandStream<R> {
    inner: R,
    max_item_size: u64,
    poisoned: bool,
}

/// Concrete implementation of the compressed item reader
pub struct ExpandItem<'d, R> {
    inner: R,
    max_item_size: u64,
    zstd: DecoderDict<'d>,
}

impl<R: Read> Expand for ExpandStream<R> {
    fn next_item(&mut self) -> Result<Option<Box<dyn Read + '_>>> {
        // this could be a panic, we don't panic in drop to assist with unwinding
        if self.poisoned {
            return Err(Error::ApiMisuse);
        }
        let mut buf = [0u8; 8];
        self.inner.read_exact(&mut buf)?;
        let len = u64::from_le_bytes(buf);
        // TODO: actually check this is a footer and not just corrupt.
        if len >= MAX_ITEM_SIZE {
            return Ok(None);
        }
        if len > self.max_item_size {
            return Err(Error::InvalidItem);
        }

        Ok(Some(Box::new(ExpandStreamItem {
            inner: self,
            limit: len,
        })))
    }
}

// Take<> but with error handling
struct ExpandStreamItem<'i, R> {
    inner: &'i mut ExpandStream<R>,
    limit: u64,
}

impl<R: Read> Read for ExpandStreamItem<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.limit == 0 {
            return Ok(0);
        }
        let max = self.limit.min(buf.len() as u64) as usize;
        let n = self.inner.inner.read(&mut buf[..max])?;
        self.limit -= n as u64;
        Ok(n)
    }
}

impl<R> Drop for ExpandStreamItem<'_, R> {
    fn drop(&mut self) {
        if self.limit != 0 {
            self.inner.poisoned = true;
        }
    }
}

impl<'d, R: BufRead> Expand for ExpandItem<'d, R> {
    fn next_item(&mut self) -> Result<Option<Box<dyn Read + '_>>> {
        let mut buf = [0u8; 8];
        self.inner.read_exact(&mut buf)?;
        let len = u64::from_le_bytes(buf);
        // TODO: actually check this is a footer and not just corrupt.
        if len >= MAX_ITEM_SIZE {
            return Ok(None);
        }
        if len > self.max_item_size {
            return Err(Error::InvalidItem);
        }

        let take = (&mut self.inner).take(len);
        let decoder = self.zstd.decode(take)?;
        Ok(Some(Box::new(decoder)))
    }
}

impl<'d> ExpandOptions<'d> {
    pub fn stream<R: BufRead + 'd>(&self, mut inner: R) -> Result<Box<dyn Expand + 'd>> {
        let hints = inner.fill_buf()?;
        assert_eq!(0x28, ZSTD_MAGIC[0]);
        assert_eq!(0x29, HEADER_TEMPLATE[0]);
        match hints[0] {
            0x28 => {
                let inner = io::BufReader::new(self.zstd.decode(inner)?);
                return self.stream(Box::new(inner) as Box<dyn BufRead + '_>);
            }
            0x29 => (),
            _ => return Err(Error::MagicMissing),
        }

        let mut buf = [0u8; 8];
        inner.read_exact(&mut buf)?;
        let max_item_size = self.max_item_size;
        let kind = parse_header(&buf)?;
        Ok(match kind {
            Kinds::Plain => Box::new(ExpandStream {
                inner,
                max_item_size,
                poisoned: false,
            }),
            Kinds::ItemCompressed => Box::new(ExpandItem {
                inner,
                max_item_size,
                zstd: self.zstd.clone(),
            }),
        })
    }
}

#[cfg(never)]
fn alloc(len: u64) -> Result<Vec<u8>> {
    let len = usize::try_from(len).map_err(|_| Error::InvalidItem)?;

    let mut buf = Vec::new();
    buf.try_reserve_exact(len)?;
    for _ in 0..len {
        buf.push(0)
    }
    Ok(buf)
}

impl<'d> ExpandOptions<'d> {
    #[must_use]
    pub fn without_dict(mut self) -> Self {
        self.zstd = DecoderDict::None;
        self
    }

    #[must_use]
    pub fn with_dict(mut self, dict: &'d DecoderDictionary<'static>) -> Self {
        self.zstd = DecoderDict::Dict(dict);
        self
    }
}
