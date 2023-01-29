use std::io;
use std::io::Read;

use archiv::{Compress, CompressOptions, Error, ExpandOptions};

fn test_round_trip<W: AsRef<[u8]> + 'static>(
    mut archiv: impl Compress<W>,
    originals: &[&str],
) -> anyhow::Result<()> {
    for item in originals {
        archiv.write_item(item.as_bytes())?;
    }
    let file = archiv.finish()?;

    let mut items = Vec::with_capacity(originals.len());
    let mut archiv = ExpandOptions::default().stream(io::Cursor::new(file))?;

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
        CompressOptions::default().stream_compress(Vec::new())?,
        &["hello world", "bruises"],
    )?;
    test_round_trip(
        CompressOptions::default().stream_compress(Vec::new())?,
        &["hello world"],
    )?;
    test_round_trip(CompressOptions::default().stream_compress(Vec::new())?, &[])?;
    Ok(())
}

#[test]
fn round_trip_items() -> anyhow::Result<()> {
    test_round_trip(
        CompressOptions::default().item_compress(Vec::new())?,
        &["hello world", "bruises"],
    )?;
    test_round_trip(
        CompressOptions::default().item_compress(Vec::new())?,
        &["hello world"],
    )?;
    test_round_trip(CompressOptions::default().item_compress(Vec::new())?, &[])?;
    Ok(())
}

#[test]
fn api_misuse() -> anyhow::Result<()> {
    let archiv = CompressOptions::default().with_level(7);
    let mut archiv = archiv.stream_compress(Vec::new())?;
    archiv.write_item(b"hello world")?;
    let out = archiv.finish()?;
    let mut archiv = ExpandOptions::default().stream(io::Cursor::new(out))?;
    let mut item = archiv.next_item()?.expect(">1 items present");
    assert_eq!(1, item.read(&mut [0u8])?);
    // this is illegal, it hasn't been fully read:
    drop(item);
    assert!(matches!(archiv.next_item(), Err(Error::ApiMisuse)));
    Ok(())
}

#[test]
fn compress_opts_lifetime() -> anyhow::Result<()> {
    let opts = CompressOptions::default().with_level(7);
    let mut archiv = opts.stream_compress(Vec::new())?;
    drop(opts);
    archiv.write_item(b"hello world")?;
    archiv.finish()?;
    Ok(())
}
