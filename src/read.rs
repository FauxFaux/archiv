use std::io;
use std::io::{BufRead, Read};

use crate::error::{Error, Result};
use crate::header::{parse_header, Kinds, HEADER_TEMPLATE, ZSTD_MAGIC};

pub struct ReadOptions {
    max_item_size: u64,
}

impl Default for ReadOptions {
    fn default() -> Self {
        const GIGABYTE: u64 = 1024 * 1024 * 1024;
        ReadOptions {
            max_item_size: 2 * GIGABYTE,
        }
    }
}

pub trait Expand {
    fn next_item(&mut self) -> Result<Option<Box<dyn Read + '_>>>;
}

pub struct StreamExpand<R> {
    inner: R,
    max_item_size: u64,
}

pub struct ItemExpand<R> {
    inner: R,
    max_item_size: u64,
}

impl<R: Read> Expand for StreamExpand<R> {
    fn next_item(&mut self) -> Result<Option<Box<dyn Read + '_>>> {
        let mut buf = [0u8; 8];
        self.inner.read_exact(&mut buf)?;
        let len = u64::from_ne_bytes(buf);
        if len >= 0xffff_fff0 {
            return Ok(None);
        }
        if len > self.max_item_size {
            return Err(Error::InvalidItem);
        }

        Ok(Some(Box::new((&mut self.inner).take(len))))
    }
}

impl<R: Read> Expand for ItemExpand<R> {
    fn next_item(&mut self) -> Result<Option<Box<dyn Read + '_>>> {
        let mut buf = [0u8; 8];
        self.inner.read_exact(&mut buf)?;
        let len = u64::from_ne_bytes(buf);
        if len >= 0xffff_fff0 {
            return Ok(None);
        }
        if len > self.max_item_size {
            return Err(Error::InvalidItem);
        }

        let take = (&mut self.inner).take(len);
        let decoder = zstd::Decoder::new(take)?;
        Ok(Some(Box::new(decoder)))
    }
}

impl ReadOptions {
    pub fn stream<R: BufRead + 'static>(self, mut inner: R) -> Result<Box<dyn Expand>> {
        let hints = inner.fill_buf()?;
        assert_eq!(0x28, ZSTD_MAGIC[0]);
        assert_eq!(0x29, HEADER_TEMPLATE[0]);
        match hints[0] {
            0x28 => {
                let inner = io::BufReader::new(zstd::Decoder::new(inner)?);
                return self.stream(Box::new(inner) as Box<dyn BufRead>);
            }
            0x29 => (),
            _ => return Err(Error::MagicMissing),
        }

        let mut buf = [0u8; 8];
        inner.read_exact(&mut buf)?;
        let max_item_size = self.max_item_size;
        let kind = parse_header(&buf)?;
        Ok(match kind {
            Kinds::Plain => Box::new(StreamExpand {
                inner,
                max_item_size,
            }),
            Kinds::ItemCompressed => Box::new(ItemExpand {
                inner,
                max_item_size,
            }),
        })
    }
}

fn alloc(len: u64) -> Result<Vec<u8>> {
    let len = usize::try_from(len).map_err(|_| Error::InvalidItem)?;

    let mut buf = Vec::new();
    buf.try_reserve_exact(len)?;
    for _ in 0..len {
        buf.push(0)
    }
    Ok(buf)
}
