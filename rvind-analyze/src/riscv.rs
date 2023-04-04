use std::{collections::HashMap, fmt};

use lazy_static::lazy_static;

#[rustfmt::skip]
pub static REG_NAMES: &[&str] = &["zero","ra","sp","gp","tp","t0","t1","t2","s0","s1","a0","a1","a2","a3","a4","a5","a6","a7","s2","s3","s4","s5","s6","s7","s8","s9","s10","s11","t3","t4","t5","t6"];

#[derive(Debug, Clone, Copy)]
pub struct Encoding {
    pub name: &'static str,
    pub mask: u32,
    pub value: u32,
    pub fields: &'static [&'static str],
}

#[derive(Clone, Copy)]
pub struct Field {
    pub decode: fn(u32) -> i64,
    pub format: fn(i64, &mut fmt::Formatter) -> fmt::Result,
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("decode", &format_args!("..."))
            .field("format", &format_args!("..."))
            .finish()
    }
}

#[rustfmt::skip]
static ENCODINGS_32_DATA: &[Encoding] = &[
    // rv_i
    Encoding { name: "lui", mask: 0x0000007f, value: 0x00000037, fields: &["rd", "imm20"] },
    Encoding { name: "auipc", mask: 0x0000007f, value: 0x00000017, fields: &["rd", "imm20"] },
    Encoding { name: "jal", mask: 0x0000007f, value: 0x0000006f, fields: &["rd", "jimm20"] },
    Encoding { name: "jalr", mask: 0x0000707f, value: 0x00000067, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "beq", mask: 0x0000707f, value: 0x00000063, fields: &["rs1", "rs2", "bimm12hilo"] },
    Encoding { name: "bne", mask: 0x0000707f, value: 0x00001063, fields: &["rs1", "rs2", "bimm12hilo"] },
    Encoding { name: "blt", mask: 0x0000707f, value: 0x00004063, fields: &["rs1", "rs2", "bimm12hilo"] },
    Encoding { name: "bge", mask: 0x0000707f, value: 0x00005063, fields: &["rs1", "rs2", "bimm12hilo"] },
    Encoding { name: "bltu", mask: 0x0000707f, value: 0x00006063, fields: &["rs1", "rs2", "bimm12hilo"] },
    Encoding { name: "bgeu", mask: 0x0000707f, value: 0x00007063, fields: &["rs1", "rs2", "bimm12hilo"] },
    Encoding { name: "lb", mask: 0x0000707f, value: 0x00000003, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "lh", mask: 0x0000707f, value: 0x00001003, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "lw", mask: 0x0000707f, value: 0x00002003, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "lbu", mask: 0x0000707f, value: 0x00004003, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "lhu", mask: 0x0000707f, value: 0x00005003, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "sb", mask: 0x0000707f, value: 0x00000023, fields: &["rs1", "rs2", "imm12hilo"] },
    Encoding { name: "sh", mask: 0x0000707f, value: 0x00001023, fields: &["rs1", "rs2", "imm12hilo"] },
    Encoding { name: "sw", mask: 0x0000707f, value: 0x00002023, fields: &["rs1", "rs2", "imm12hilo"] },
    Encoding { name: "addi", mask: 0x0000707f, value: 0x00000013, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "slti", mask: 0x0000707f, value: 0x00002013, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "sltiu", mask: 0x0000707f, value: 0x00003013, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "xori", mask: 0x0000707f, value: 0x00004013, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "ori", mask: 0x0000707f, value: 0x00006013, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "andi", mask: 0x0000707f, value: 0x00007013, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "add", mask: 0xfe00707f, value: 0x00000033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "sub", mask: 0xfe00707f, value: 0x40000033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "sll", mask: 0xfe00707f, value: 0x00001033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "slt", mask: 0xfe00707f, value: 0x00002033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "sltu", mask: 0xfe00707f, value: 0x00003033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "xor", mask: 0xfe00707f, value: 0x00004033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "srl", mask: 0xfe00707f, value: 0x00005033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "sra", mask: 0xfe00707f, value: 0x40005033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "or", mask: 0xfe00707f, value: 0x00006033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "and", mask: 0xfe00707f, value: 0x00007033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "fence", mask: 0x0000707f, value: 0x0000000f, fields: &["rd", "rs1", "fm", "pred", "succ"] },
    Encoding { name: "ecall", mask: 0xffffffff, value: 0x00000073, fields: &[] },
    Encoding { name: "ebreak", mask: 0xffffffff, value: 0x00100073, fields: &[] },

    // rv64_i
    Encoding { name: "lwu", mask: 0x0000707f, value: 0x00006003, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "ld", mask: 0x0000707f, value: 0x00003003, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "sd", mask: 0x0000707f, value: 0x00003023, fields: &["rs1", "rs2", "imm12hilo"] },
    Encoding { name: "slli", mask: 0xfc00707f, value: 0x00001013, fields: &["rd", "rs1", "shamtd"] },
    Encoding { name: "srli", mask: 0xfc00707f, value: 0x00005013, fields: &["rd", "rs1", "shamtd"] },
    Encoding { name: "srai", mask: 0xfc00707f, value: 0x40005013, fields: &["rd", "rs1", "shamtd"] },
    Encoding { name: "addiw", mask: 0x0000707f, value: 0x0000001b, fields: &["rd", "rs1", "imm12"] },
    Encoding { name: "slliw", mask: 0xfe00707f, value: 0x0000101b, fields: &["rd", "rs1", "shamtw"] },
    Encoding { name: "srliw", mask: 0xfe00707f, value: 0x0000501b, fields: &["rd", "rs1", "shamtw"] },
    Encoding { name: "sraiw", mask: 0xfe00707f, value: 0x4000501b, fields: &["rd", "rs1", "shamtw"] },
    Encoding { name: "addw", mask: 0xfe00707f, value: 0x0000003b, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "subw", mask: 0xfe00707f, value: 0x4000003b, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "sllw", mask: 0xfe00707f, value: 0x0000103b, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "srlw", mask: 0xfe00707f, value: 0x0000503b, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "sraw", mask: 0xfe00707f, value: 0x4000503b, fields: &["rd", "rs1", "rs2"] },
];

fn format_value(value: i64, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{value}")
}

fn format_reg(value: i64, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", REG_NAMES[value as usize])
}

fn uf(v: u32, s: u32, l: u32) -> i64 {
    ((v >> s) & ((1 << l) - 1)) as i64
}

fn sf(v: u32, s: u32, l: u32) -> i64 {
    (v as i64) << (64 - s - l) >> (64 - l)
}

#[rustfmt::skip]
static FIELDS_DATA: &[(&str, Field)] = &[
    ("rd", Field { format: format_reg, decode: |v| uf(v,7,5) }),
    ("rs1", Field { format: format_reg, decode: |v| uf(v,15,5) }),
    ("rs2", Field { format: format_reg, decode: |v| uf(v,20,5) }),

    ("imm20", Field { format: format_value, decode: |v| sf(v,12,20) << 12 }),
    ("jimm20", Field { format: format_value, decode: |v| (uf(v,21,10) << 1) | (uf(v,20,1) << 11) | (uf(v,12,8) << 12) | (sf(v,31,1) << 20) }),
    ("imm12", Field { format: format_value, decode: |v| sf(v,20,12) }),
    ("imm12hilo", Field { format: format_value, decode: |v| uf(v,7,5) | (sf(v,25,7) << 5) }),
    ("bimm12hilo", Field { format: format_value, decode: |v| (uf(v,8,4) << 1) | (uf(v,25,6) << 5) | (uf(v,7,1) << 11) | (sf(v,31,1) << 12) }),
    ("shamtd", Field { format: format_value, decode: |v| uf(v,20,6) }),
    ("shamtw", Field { format: format_value, decode: |v| uf(v,20,5) }),

    // ("fm", Field { format: format_value, decode: |v| todo!() }),
    // ("pred", Field { format: format_value, decode: |v| todo!() }),
    // ("succ", Field { format: format_value, decode: |v| todo!() }),
];

lazy_static! {
    static ref ENCODINGS_32: HashMap<u32, Vec<Encoding>> = {
        let mut res: HashMap<u32, Vec<Encoding>> = HashMap::new();

        for enc in ENCODINGS_32_DATA {
            assert!(enc.mask & 0x7f == 0x7f);
            res.entry(enc.value & 0x7f).or_default().push(*enc);
        }

        res
    };
    static ref FIELDS: HashMap<&'static str, Field> = FIELDS_DATA.iter().copied().collect();
}

pub fn decode(insn: u32) -> Option<Encoding> {
    if insn & 0b11 == 0b11 {
        // 32b insn
        ENCODINGS_32[&(insn & 0x7f)]
            .iter()
            .find(|enc| (insn & enc.mask) == enc.value)
            .copied()
    } else {
        // 16b insn, TODO
        None
    }
}

pub fn field(name: &str) -> Field {
    FIELDS[name]
}

#[derive(Debug)]
pub struct Disassembly {
    insn: u32,
    encoding: Option<Encoding>,
}

impl fmt::Display for Disassembly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.insn & 0b11 == 0b11 {
            write!(f, "{:08x} ", self.insn)?;
        } else {
            write!(f, "{:04x}     ", self.insn)?;
        }

        if let Some(encoding) = self.encoding {
            let mut first = true;

            write!(f, "{:7}", encoding.name)?;
            for &name in encoding.fields {
                let fld = field(name);
                let field = (fld.decode)(self.insn);
                if first {
                    write!(f, " ")?;
                    first = false;
                } else {
                    write!(f, ", ")?;
                }
                (fld.format)(field, f)?;
            }
        } else {
            write!(f, "?")?;
        }
        Ok(())
    }
}

pub fn disassemble(insn: u32) -> Disassembly {
    Disassembly {
        insn,
        encoding: decode(insn),
    }
}
