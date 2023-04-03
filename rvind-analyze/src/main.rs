mod elf;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use elf::Executable;
use std::{ffi::OsString, fs};

#[derive(Parser, Debug)]
struct Args {
    file: OsString,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = &args.file;
    let buf = fs::read(file).context(anyhow!("Cannot read binary file {file:?}"))?;
    println!(
        "{:#x?}",
        Executable::from_bytes(&buf).context(anyhow!("Failed to parse file {file:?}"))?
    );

    Ok(())
}
