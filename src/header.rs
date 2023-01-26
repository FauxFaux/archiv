use crate::error::{Error, Result};

//                      (all other values reserved)
//                           Kinds enum   ----v
const HEADER_TEMPLATE: [u8; 8] = *b"arch\0\0\0\0";
const FOOTER_TEMPLATE: [u8; 8] = u64::to_ne_bytes(0xffff_fff0);
pub const GLOBAL_MARKER_LEN: u64 = 8;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Kinds {
    StreamCompressed = 0,
    ItemCompressed = 1,
}

pub fn header(kind: Kinds) -> [u8; 8] {
    let mut header = HEADER_TEMPLATE;
    header[7] = kind as u8;
    header
}

pub fn parse_header(buf: &[u8; 8]) -> Result<Kinds> {
    if buf[..7] != HEADER_TEMPLATE[..7] {
        return Err(Error::MagicMissing);
    }

    Ok(match buf[7] {
        0 => Kinds::StreamCompressed,
        1 => Kinds::ItemCompressed,
        _ => return Err(Error::MagicUnrecognised),
    })
}

pub fn footer() -> [u8; 8] {
    FOOTER_TEMPLATE
}
