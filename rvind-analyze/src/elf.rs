use anyhow::{bail, Result};
use goblin::{container::Ctx, elf, strtab::Strtab};
use std::{collections::HashMap, ops::Range};

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub section: usize,
    pub addr: u64,
    pub size: u64,
}

impl Symbol {
    fn from(sym: &elf::Sym, strtab: &Strtab) -> Self {
        Self {
            name: strtab.get_at(sym.st_name).unwrap().to_owned(),
            section: sym.st_shndx,
            addr: sym.st_value,
            size: sym.st_size,
        }
    }
}

#[derive(Debug)]
pub struct Relocation {
    pub symbol: Symbol,
    pub ty: u32,
    pub addend: Option<i64>,
}

impl Relocation {
    fn from(symtab: &elf::Symtab, strtab: &Strtab, reloc: &elf::Reloc) -> Self {
        Self {
            symbol: Symbol::from(&symtab.get(reloc.r_sym).unwrap(), strtab),
            ty: reloc.r_type,
            addend: reloc.r_addend,
        }
    }
}

#[derive(Debug)]
pub struct Section {
    pub name: String,
    pub data: Range<usize>,
    pub addr: u64,
    pub relocations: HashMap<u64, Vec<Relocation>>,
}

impl Section {
    fn from(elf: &elf::Elf, sh: &elf::SectionHeader) -> Self {
        let name = elf.shdr_strtab.get_at(sh.sh_name).unwrap();
        Self {
            name: name.to_owned(),
            data: sh.file_range().unwrap_or(0..0),
            addr: sh.sh_addr,
            relocations: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Executable {
    pub sections: Vec<Section>,
    pub functions: Vec<Symbol>,
}

fn elf_context(elf: &elf::Elf) -> Ctx {
    use goblin::container::*;
    Ctx {
        container: if elf.is_64 {
            Container::Big
        } else {
            Container::Little
        },
        le: if elf.little_endian {
            Endian::Little
        } else {
            Endian::Big
        },
    }
}

impl Executable {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let elf = elf::Elf::parse(bytes)?;
        let ctx = elf_context(&elf);

        let mut sections: Vec<Section> = elf
            .section_headers
            .iter()
            .map(|sh| Section::from(&elf, sh))
            .collect();

        for (i, reloc_section) in &elf.shdr_relocs {
            let i = *i;
            let sh = &elf.section_headers[i];
            let sec = &mut sections[sh.sh_info as usize];
            let target = &mut sec.relocations;

            if !target.is_empty() {
                bail!("Multiple relocations for the same section {}", sec.name);
            }

            let symtab_sh = &elf.section_headers[sh.sh_link as usize];
            let strtab_sh = &elf.section_headers[symtab_sh.sh_link as usize];

            let sym_size = elf::Sym::size(ctx.container);

            if symtab_sh.sh_size as usize % sym_size != 0 {
                bail!(
                    "Invalid symtab size {}, not a multiple of {sym_size}",
                    symtab_sh.sh_size
                );
            }

            let symtab = elf::Symtab::parse(
                bytes,
                symtab_sh.sh_offset as usize,
                symtab_sh.sh_size as usize / sym_size,
                ctx,
            )?;

            let strtab = Strtab::parse(
                bytes,
                strtab_sh.sh_offset as usize,
                strtab_sh.sh_size as usize,
                0,
            )?;

            for reloc in reloc_section {
                target
                    .entry(reloc.r_offset)
                    .or_insert(Vec::new())
                    .push(Relocation::from(&symtab, &strtab, &reloc));
            }
        }

        let functions = elf
            .syms
            .iter()
            .filter(|sym| sym.is_function() && sym.st_shndx != 0 && sym.st_size != 0)
            .map(|sym| Symbol::from(&sym, &elf.strtab))
            .collect();

        Ok(Self {
            sections,
            functions,
        })
    }
}
