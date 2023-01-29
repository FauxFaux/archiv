use crate::error::{Error, Result};

//                          (all other values reserved)
//                                Kinds enum   ----v
pub const HEADER_TEMPLATE: [u8; 8] = *b"\x29\xb6arc\0\0\0";
const FOOTER_TEMPLATE: [u8; 8] = u64::to_le_bytes(0xffff_ffff_ffff_fff0);
pub const GLOBAL_MARKER_LEN: u64 = 8;

// 2^63.9 bytes, over 17 million terabytes.
// (obviously this is ridiculous)
pub const MAX_ITEM_SIZE: u64 = 0xf000_0000_0000_0000;

pub const ZSTD_MAGIC: [u8; 4] = *b"\x28\xb5\x2f\xfd";

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Kinds {
    Plain = 0,
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
        0 => Kinds::Plain,
        1 => Kinds::ItemCompressed,
        _ => return Err(Error::MagicUnrecognised),
    })
}

pub fn footer() -> [u8; 8] {
    FOOTER_TEMPLATE
}
