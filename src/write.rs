use std::io::Write;

use crate::error::{Error, Result};
use crate::header::{footer, header, Kinds, GLOBAL_MARKER_LEN};

pub struct WriteOptions {
    level: i32,
}

impl Default for WriteOptions {
    fn default() -> Self {
        WriteOptions { level: 3 }
    }
}

pub struct Plain<W: Write> {
    off: u64,
    inner: zstd::Encoder<'static, W>,
}

pub struct ItemCompress<W> {
    off: u64,
    level: i32,
    inner: W,
}

impl<W: Write> Plain<W> {
    pub fn write_item(&mut self, item: &[u8]) -> Result<u64> {
        let len = u64::try_from(item.len()).map_err(|_| Error::LengthOverflow)?;
        self.inner.write_all(&len.to_ne_bytes())?;
        self.inner.write_all(item)?;
        self.off = self.off.checked_add(len).ok_or(Error::LengthOverflow)?;
        Ok(self.off)
    }

    pub fn write_item_vectored(&mut self, item: &[&[u8]]) -> Result<u64> {
        let mut len: u64 = 0;
        for slice in item {
            len = len
                .checked_add(u64::try_from(slice.len()).map_err(|_| Error::LengthOverflow)?)
                .ok_or(Error::LengthOverflow)?;
        }
        self.inner.write_all(&len.to_ne_bytes())?;
        for slice in item {
            self.inner.write_all(slice)?;
        }
        let start = self.off;
        self.off = self
            .off
            .checked_add(GLOBAL_MARKER_LEN + len)
            .ok_or(Error::LengthOverflow)?;
        Ok(start)
    }

    pub fn get_mut(&mut self) -> &mut W {
        self.inner.get_mut()
    }

    pub fn finish(mut self) -> Result<W> {
        self.inner.write_all(&footer())?;
        let mut w = self.inner.finish()?;
        w.flush()?;
        Ok(w)
    }
}

impl<W: Write> ItemCompress<W> {
    pub fn write_item(&mut self, item: &[u8]) -> Result<u64> {
        let original_len = u64::try_from(item.len()).map_err(|_| Error::LengthOverflow)?;
        let mut buf = Vec::with_capacity(item.len() / 4 + 30);
        let mut writer = zstd::Encoder::new(&mut buf, self.level)?;
        writer.set_pledged_src_size(Some(original_len))?;
        writer.include_contentsize(true)?;
        writer.write_all(item)?;
        writer.finish()?;

        let new_len = u64::try_from(buf.len()).map_err(|_| Error::LengthOverflow)?;
        self.inner.write_all(&new_len.to_ne_bytes())?;
        self.inner.write_all(&buf)?;
        let start = self.off;
        self.off = self
            .off
            .checked_add(GLOBAL_MARKER_LEN + new_len)
            .ok_or(Error::LengthOverflow)?;
        Ok(start)
    }

    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    pub fn finish(self) -> Result<W> {
        let mut w = self.inner;
        w.write_all(&footer())?;
        w.flush()?;
        Ok(w)
    }
}

impl WriteOptions {
    pub fn stream_compress<W: Write>(self, inner: W) -> Result<Plain<W>> {
        let mut inner = zstd::Encoder::new(inner, self.level)?;
        inner.write_all(&header(Kinds::Plain))?;
        Ok(Plain {
            off: GLOBAL_MARKER_LEN,
            inner,
        })
    }

    pub fn item_compress<W: Write>(self, mut inner: W) -> Result<ItemCompress<W>> {
        inner.write_all(&header(Kinds::ItemCompressed))?;
        Ok(ItemCompress {
            off: GLOBAL_MARKER_LEN,
            level: self.level,
            inner,
        })
    }
}
