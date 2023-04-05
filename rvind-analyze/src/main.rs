mod elf;
mod riscv;
mod analysis;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use elf::Executable;
use std::{ffi::OsString, fs, collections::HashMap};

#[derive(Parser, Debug)]
struct Args {
    file: OsString,
}

fn disassemble(mut addr: i64, mut bytes: &[u8], states: &HashMap<i64, analysis::AbstractState>) {
    let range = addr..addr + (bytes.len() as i64);

    while let Some(first) = bytes.first() {
        let (ilen, insn) = if first & 0b11 == 0b11 {
            (4, u32::from_le_bytes(bytes[..4].try_into().unwrap()))
        } else {
            (2, u16::from_le_bytes(bytes[..2].try_into().unwrap()) as u32)
        };

        let analysis = analysis::analyze_insn(addr, &range, insn);

        if let Some(state) = states.get(&addr) {
            println!("{state}");
            state.check(&analysis);
        } else {
            println!("<unreachable?>");
        }
        println!("  {addr:>#10x}: {}", riscv::disassemble(insn));
        println!("  {:>10}  = {analysis}", "");
        println!();

        addr += ilen as i64;
        bytes = &bytes[ilen..];
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = &args.file;
    let buf = fs::read(file).context(anyhow!("Cannot read binary file {file:?}"))?;
    let exe = Executable::from_bytes(&buf).context(anyhow!("Failed to parse file {file:?}"))?;
    // println!("{:?}", exe);

    for f in &exe.functions {
        let sec = &exe.sections[f.section];
        let off = f.addr - sec.addr;
        let bytes = &buf[sec.data.clone()][off as usize..(off + f.size) as usize];
        println!("{}:", f.name);
        let state_map = analysis::analyze(f.addr as i64, bytes);
        disassemble(f.addr as i64, bytes, &state_map);
    }

    Ok(())
}
