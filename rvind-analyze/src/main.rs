mod analysis;
mod elf;
mod riscv;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use elf::Executable;
use std::{collections::{HashMap, BTreeMap}, ffi::OsString, fs};

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
            if let Some(uw) = state.unwind_step() {
                println!("Unwind: {uw}");
            } else {
                println!("Unwind: (Cannot unwind!)");
            }
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

struct UnwindRange {
    start: i64,
    end: i64,
    unwind: analysis::UnwindStep,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = &args.file;
    let buf = fs::read(file).context(anyhow!("Cannot read binary file {file:?}"))?;
    let exe = Executable::from_bytes(&buf).context(anyhow!("Failed to parse file {file:?}"))?;

    let mut unwind_ranges: Vec<UnwindRange> = Vec::new();
    let mut seen_functions: BTreeMap<u64, elf::Symbol> = BTreeMap::new();

    for f in &exe.functions {
        if let Some(seen) = seen_functions.get(&f.addr) {
            assert!(f.size == seen.size);
            continue;
        } else {
            seen_functions.insert(f.addr, f.clone());
        }
        let sec = &exe.sections[f.section];
        let off = f.addr - sec.addr;
        let bytes = &buf[sec.data.clone()][off as usize..(off + f.size) as usize];
        let state_map = analysis::analyze(f.addr as i64, bytes);
        for (addr, state) in state_map {
            if let Some(unwind) = state.unwind_step() {
                let insn_len = if bytes[(addr - f.addr as i64) as usize] & 0b11 == 0b11 {
                    4
                } else {
                    2
                };

                unwind_ranges.push(UnwindRange {
                    start: addr,
                    end: addr + insn_len,
                    unwind,
                });
            }
        }
    }

    unwind_ranges.sort_unstable_by_key(|r| r.start);

    let mut merged: Vec<(i64, Option<analysis::UnwindStep>)> = Vec::new();
    let mut current: i64 = unwind_ranges.first().unwrap().start;

    for UnwindRange { start, end, unwind } in unwind_ranges {
        use std::cmp::Ordering::*;

        match current.cmp(&start) {
            Less => {
                merged.push((current, None));
                merged.push((start, Some(unwind)));
                current = end;
            }
            Equal => {
                if let Some((_, Some(last))) = merged.last() {
                    if &unwind != last {
                        merged.push((start, Some(unwind)));
                    }
                } else {
                    merged.push((start, Some(unwind)));
                }
                current = end;
            }
            Greater => panic!("Overlapping ranges"),
        }
    }

    merged.push((current, None));

    for (addr, unwind) in merged {
        if let Some(unwind) = unwind {
            println!("{addr:#x} {unwind}");
        } else {
            println!("{addr:#x} -");
        }
    }

    Ok(())
}
