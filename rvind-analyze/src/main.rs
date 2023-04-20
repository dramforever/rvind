mod analysis;
mod elf;
mod format;
mod riscv;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use elf::Executable;
use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsString,
    fs,
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short)]
    output: OsString,
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

    let (text_index, text_section) = exe
        .sections
        .iter()
        .enumerate()
        .find(|(_, s)| s.name == ".text")
        .ok_or(anyhow!("No .text section found"))?;

    let mut unwind_ranges: Vec<UnwindRange> = Vec::new();
    let mut seen_functions: BTreeMap<u64, elf::Symbol> = BTreeMap::new();

    for f in &exe.functions {
        if let Some(seen) = seen_functions.get(&f.addr) {
            assert!(f.size == seen.size);
            continue;
        } else {
            seen_functions.insert(f.addr, f.clone());
        }

        if f.section != text_index {
            eprintln!("Function {} not in .text section", f.name);
            continue;
        }

        let sec = &exe.sections[f.section];
        let off = (f.addr - sec.addr) as i64;
        let bytes = &buf[sec.data.clone()][off as usize..(off + f.size as i64) as usize];
        let state_map = analysis::analyze(f.addr as i64, bytes);
        println!("{}:", f.name);
        disassemble(f.addr as i64, bytes, &state_map);

        for (addr, state) in state_map {
            if let Some(unwind) = state.unwind_step() {
                let insn_len = if bytes[(addr - f.addr as i64) as usize] & 0b11 == 0b11 {
                    4
                } else {
                    2
                };
                unwind_ranges.push(UnwindRange {
                    start: addr - sec.addr as i64,
                    end: addr - sec.addr as i64 + insn_len,
                    unwind,
                });
            }
        }
    }

    unwind_ranges.sort_unstable_by_key(|r| r.start);

    let mut merged: Vec<(i64, Option<analysis::UnwindStep>)> = Vec::new();
    let mut current: i64 = 0;

    for UnwindRange { start, end, unwind } in unwind_ranges {
        use std::cmp::Ordering::*;

        match current.cmp(&start) {
            Less => {
                let data = &buf[text_section.data.clone()][current as usize..start as usize];

                if data.len() > 6 {
                    eprintln!(
                        "Cannot unwind at {:#x?}, {} bytes",
                        current + text_section.addr as i64,
                        data.len()
                    );
                }

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

    for (start, unwind) in &merged {
        if let Some(unwind) = unwind {
            println!("{start:#x} {unwind}");
        } else {
            println!("{start:#x} -");
        }
    }

    let mut unwind_data: Vec<u8> = Vec::new();

    for (start, unwind) in merged {
        let data = format::convert_unwind(start, unwind);
        unwind_data.extend(data.to_bytes());
    }

    fs::write(args.output, unwind_data)?;

    Ok(())
}
