use std::io;
use std::io::Read;

use archiv::{Encoder, ReadOptions, WriteOptions};

fn test_round_trip<W: AsRef<[u8]> + 'static>(
    mut archiv: impl Encoder<W>,
    originals: &[&str],
) -> anyhow::Result<()> {
    for item in originals {
        archiv.write_item(item.as_bytes())?;
    }
    let file = archiv.finish()?;

    let mut items = Vec::with_capacity(originals.len());
    let mut archiv = ReadOptions::default().stream(io::Cursor::new(file))?;

    while let Some(mut v) = archiv.next_item()? {
        let mut buf = String::with_capacity(16);
        v.read_to_string(&mut buf)?;
        items.push(buf);
    }

    assert_eq!(originals, items);
    Ok(())
}

#[test]
fn round_trip_stream() -> anyhow::Result<()> {
    test_round_trip(
        WriteOptions::default().stream_compress(Vec::new())?,
        &["hello world", "bruises"],
    )?;
    test_round_trip(
        WriteOptions::default().stream_compress(Vec::new())?,
        &["hello world"],
    )?;
    test_round_trip(WriteOptions::default().stream_compress(Vec::new())?, &[])?;
    Ok(())
}

#[test]
fn round_trip_items() -> anyhow::Result<()> {
    test_round_trip(
        WriteOptions::default().item_compress(Vec::new())?,
        &["hello world", "bruises"],
    )?;
    test_round_trip(
        WriteOptions::default().item_compress(Vec::new())?,
        &["hello world"],
    )?;
    test_round_trip(WriteOptions::default().item_compress(Vec::new())?, &[])?;
    Ok(())
}
