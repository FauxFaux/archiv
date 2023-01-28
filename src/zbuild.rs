use std::io::{BufRead, Read, Write};

use zstd::dict::{DecoderDictionary, EncoderDictionary};

use crate::error::Result;

#[derive(Default, Clone)]
pub enum ZstdDict<P> {
    #[default]
    None,
    Copy(Vec<u8>),
    Prepared(P),
}

pub struct ZstdBuilder {
    pub level: i32,
    pub dict: ZstdDict<EncoderDictionary<'static>>,
}

impl Default for ZstdBuilder {
    fn default() -> Self {
        Self {
            level: 3,
            dict: ZstdDict::None,
        }
    }
}

impl ZstdBuilder {
    pub fn encode<W: Write>(&self, inner: W) -> Result<zstd::Encoder<W>> {
        Ok(match &self.dict {
            ZstdDict::None => zstd::Encoder::new(inner, self.level)?,
            ZstdDict::Copy(v) => zstd::Encoder::with_dictionary(inner, self.level, v)?,
            ZstdDict::Prepared(p) => zstd::Encoder::with_prepared_dictionary(inner, p)?,
        })
    }
}

impl ZstdDict<DecoderDictionary<'static>> {
    pub fn decode<R: BufRead>(&self, inner: R) -> Result<zstd::Decoder<R>> {
        Ok(match self {
            ZstdDict::None => zstd::Decoder::with_buffer(inner)?,
            ZstdDict::Copy(v) => zstd::Decoder::with_dictionary(inner, v)?,
            ZstdDict::Prepared(p) => zstd::Decoder::with_prepared_dictionary(inner, p)?,
        })
    }
}
