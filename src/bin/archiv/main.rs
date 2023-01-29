mod train;

use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};

use anyhow::{anyhow, Context, Result};
use archiv::{Encoder, WriteOptions};
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
    let opts = WriteOptions::default();
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
