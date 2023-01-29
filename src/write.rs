use std::io::Write;
use zstd::dict::EncoderDictionary;

use crate::error::{Error, Result};
use crate::header::{footer, header, Kinds, GLOBAL_MARKER_LEN};
use crate::zbuild::{ZstdBuilder, ZstdDict};

#[derive(Default)]
pub struct WriteOptions<'d> {
    zstd: ZstdBuilder<'d>,
}

pub trait Encoder<W> {
    fn write_item(&mut self, item: &[u8]) -> Result<u64>;
    fn finish(self) -> Result<W>;
}

pub struct Plain<'e, W: Write> {
    off: u64,
    inner: zstd::Encoder<'e, W>,
}

pub struct ItemCompress<'d, W> {
    off: u64,
    inner: W,
    zstd: ZstdBuilder<'d>,
}

impl<'e, W: Write> Encoder<W> for Plain<'e, W> {
    fn write_item(&mut self, item: &[u8]) -> Result<u64> {
        let len = u64::try_from(item.len()).map_err(|_| Error::LengthOverflow)?;
        self.inner.write_all(&len.to_ne_bytes())?;
        self.inner.write_all(item)?;
        self.off = self.off.checked_add(len).ok_or(Error::LengthOverflow)?;
        Ok(self.off)
    }

    fn finish(mut self) -> Result<W> {
        self.inner.write_all(&footer())?;
        let mut w = self.inner.finish()?;
        w.flush()?;
        Ok(w)
    }
}

impl<'e, W: Write> Plain<'e, W> {
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
}

impl<'d, W: Write> Encoder<W> for ItemCompress<'d, W> {
    fn write_item(&mut self, item: &[u8]) -> Result<u64> {
        let original_len = u64::try_from(item.len()).map_err(|_| Error::LengthOverflow)?;
        let mut buf = Vec::with_capacity(item.len() / 4 + 30);
        let mut writer = self.zstd.encode(&mut buf)?;
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

    fn finish(self) -> Result<W> {
        let mut w = self.inner;
        w.write_all(&footer())?;
        w.flush()?;
        Ok(w)
    }
}

impl<'d, W: Write> ItemCompress<'d, W> {
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }
}

impl<'d> WriteOptions<'d> {
    pub fn stream_compress<W: Write>(&self, inner: W) -> Result<Plain<W>> {
        let mut inner = self.zstd.encode(inner)?;
        inner.write_all(&header(Kinds::Plain))?;
        Ok(Plain {
            off: GLOBAL_MARKER_LEN,
            inner,
        })
    }

    pub fn item_compress<W: Write>(&self, mut inner: W) -> Result<ItemCompress<'d, W>> {
        inner.write_all(&header(Kinds::ItemCompressed))?;
        Ok(ItemCompress {
            off: GLOBAL_MARKER_LEN,
            inner,
            zstd: self.zstd.clone(),
        })
    }
}

impl<'d> WriteOptions<'d> {
    #[must_use]
    pub fn with_level(mut self, val: i32) -> Self {
        self.zstd.level = val;
        self
    }

    #[must_use]
    pub fn without_dictionary(mut self) -> Self {
        self.zstd.dict = ZstdDict(None);
        self
    }

    #[must_use]
    pub fn with_dict(mut self, dict: &'d EncoderDictionary<'static>) -> Self {
        self.zstd.dict = ZstdDict(Some(dict));
        self
    }
}
