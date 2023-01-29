use std::io::{BufRead, Write};

use zstd::dict::{DecoderDictionary, EncoderDictionary};

use crate::error::Result;

pub struct ZstdDict<'d, D>(pub Option<&'d D>);

impl<'d, D> Clone for ZstdDict<'d, D> {
    fn clone(&self) -> Self {
        ZstdDict(self.0.clone())
    }
}

impl Default for ZstdDict<'static, DecoderDictionary<'static>> {
    fn default() -> Self {
        Self(None)
    }
}

#[derive(Clone)]
pub struct ZstdBuilder<'d> {
    pub level: i32,
    pub dict: ZstdDict<'d, EncoderDictionary<'static>>,
}

impl Default for ZstdBuilder<'static> {
    fn default() -> Self {
        Self {
            level: 3,
            dict: ZstdDict(None),
        }
    }
}

impl<'d> ZstdBuilder<'d> {
    pub fn encode<W: Write>(&self, inner: W) -> Result<zstd::Encoder<W>> {
        Ok(match self.dict.0 {
            None => zstd::Encoder::new(inner, self.level)?,
            Some(p) => zstd::Encoder::with_prepared_dictionary(inner, p)?,
        })
    }
}

impl<'d> ZstdDict<'d, DecoderDictionary<'static>> {
    pub fn decode<R: BufRead>(&self, inner: R) -> Result<zstd::Decoder<'d, R>> {
        Ok(match self.0 {
            None => zstd::Decoder::with_buffer(inner)?,
            Some(p) => zstd::Decoder::with_prepared_dictionary(inner, p)?,
        })
    }
}
