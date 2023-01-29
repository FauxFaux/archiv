use std::io::{BufRead, Write};

use zstd::dict::{DecoderDictionary, EncoderDictionary};

use crate::error::Result;

#[derive(Clone)]
pub enum EncoderDict<'d> {
    None(i32),
    Dict(&'d EncoderDictionary<'static>),
}

#[derive(Clone, Default)]
pub enum DecoderDict<'d> {
    #[default]
    None,
    Dict(&'d DecoderDictionary<'static>),
}

impl<'d> EncoderDict<'d> {
    pub fn encode<W: Write>(&self, inner: W) -> Result<zstd::Encoder<W>> {
        Ok(match self {
            EncoderDict::None(level) => zstd::Encoder::new(inner, *level)?,
            EncoderDict::Dict(p) => zstd::Encoder::with_prepared_dictionary(inner, p)?,
        })
    }
}

impl<'d> DecoderDict<'d> {
    pub fn decode<R: BufRead>(&self, inner: R) -> Result<zstd::Decoder<'d, R>> {
        Ok(match self {
            DecoderDict::None => zstd::Decoder::with_buffer(inner)?,
            DecoderDict::Dict(p) => zstd::Decoder::with_prepared_dictionary(inner, p)?,
        })
    }
}

impl Default for EncoderDict<'_> {
    fn default() -> Self {
        EncoderDict::None(0)
    }
}
