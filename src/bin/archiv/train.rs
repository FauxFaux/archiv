use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};

use anyhow::Result;
use archiv::ExpandOptions;

pub fn train(sources: &[PathBuf], limit: usize) -> Result<Vec<u8>> {
    let mut samples = Vec::with_capacity(limit);
    let mut i = 0usize;
    eprintln!("Loading samples...");
    for source in sources {
        let opts = ExpandOptions::default();
        let mut v = opts.stream(io::BufReader::new(fs::File::open(source)?))?;
        // TODO: skip decompression on item-compressed archives, where possible
        while let Some(mut item) = v.next_item()? {
            let mut buf = Vec::with_capacity(4 * 1024);
            item.read_to_end(&mut buf)?;
            i = i.wrapping_add(buf.len()).wrapping_mul(37);
            if samples.len() < limit {
                samples.push(buf);
            } else {
                assert_eq!(samples.len(), limit);
                samples[i % limit] = buf;
            }
        }
    }
    eprintln!("Training...");
    Ok(zstd::dict::from_samples(&samples, 112640)?)
}
