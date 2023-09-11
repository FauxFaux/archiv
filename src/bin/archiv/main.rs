mod train;

use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};

use anyhow::{anyhow, Context, Result};
use archiv::{Compress, CompressOptions, ExpandOptions};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Dump the contents of files into a single archiv, written to stdout
    Pack {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    Stats {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// Build a dictionary from documents 'randomly' selected from source archive(s)
    Train {
        /// archivs to read source documents from
        #[arg(required = true)]
        sources: Vec<PathBuf>,

        /// Path to write the resulting dictionary to
        #[arg(short, long, default_value = "dictionary")]
        out: PathBuf,

        /// Maximum number of documents to train on
        #[arg(short, long, default_value = "10000")]
        limit: usize,
    },
}

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();
    match cli.command {
        Commands::Pack { files } => pack(&files)?,
        Commands::Stats { files } => stats(&files)?,
        Commands::Train {
            sources,
            out,
            limit,
        } => {
            let dict = train::train(&sources, limit)?;
            fs::write(out, dict)?;
        }
    }
    Ok(())
}

fn pack(files: &[PathBuf]) -> Result<()> {
    let stdout = io::stdout().lock();
    let opts = CompressOptions::default();
    let mut archiv = opts.stream_compress(stdout)?;
    let mut buf = Vec::with_capacity(4096);
    for file in files {
        buf.clear();
        let mut file = fs::File::open(file).with_context(|| anyhow!("{file:?}"))?;
        file.read_to_end(&mut buf)?;
        archiv.write_item(&buf)?;
    }
    let _ = archiv.finish()?;
    Ok(())
}

fn stats(files: &[PathBuf]) -> Result<()> {
    for file in files {
        print!("{}: ", file.display());
        let file = fs::File::open(file).with_context(|| anyhow!("{file:?}"))?;
        let file = io::BufReader::new(file);
        let mut file = ExpandOptions::default().stream(file)?;
        let mut count = 0u64;
        let mut bytes = 0u64;
        let mut buf = Vec::with_capacity(4096);
        while let Some(mut item) = file.next_item()? {
            buf.clear();
            item.read_to_end(&mut buf)?;
            bytes += u64::try_from(buf.len())?;
            count += 1;
        }
        println!(
            "{} items, {} bytes, {} mean size",
            count,
            bytes,
            bytes / count
        );
    }
    Ok(())
}
