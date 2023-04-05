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
    Encoding { name: "fence", mask: 0x0000707f, value: 0x0000000f, fields: &[] /* &["rd", "rs1", "fm", "pred", "succ"] */ },
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

    // rv_m
    Encoding { name: "mul", mask: 0xfe00707f, value: 0x02000033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "mulh", mask: 0xfe00707f, value: 0x02001033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "mulhsu", mask: 0xfe00707f, value: 0x02002033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "mulhu", mask: 0xfe00707f, value: 0x02003033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "div", mask: 0xfe00707f, value: 0x02004033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "divu", mask: 0xfe00707f, value: 0x02005033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "rem", mask: 0xfe00707f, value: 0x02006033, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "remu", mask: 0xfe00707f, value: 0x02007033, fields: &["rd", "rs1", "rs2"] },

    // rv64_m
    Encoding { name: "mulw", mask: 0xfe00707f, value: 0x0200003b, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "divw", mask: 0xfe00707f, value: 0x0200403b, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "divuw", mask: 0xfe00707f, value: 0x0200503b, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "remw", mask: 0xfe00707f, value: 0x0200603b, fields: &["rd", "rs1", "rs2"] },
    Encoding { name: "remuw", mask: 0xfe00707f, value: 0x0200703b, fields: &["rd", "rs1", "rs2"] },

    // rv_a
    Encoding { name: "lr.w", mask: 0xf9f0707f, value: 0x1000202f, fields: &["rd", "rs1", /* "aq", "rl" */] },
    Encoding { name: "sc.w", mask: 0xf800707f, value: 0x1800202f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amoswap.w", mask: 0xf800707f, value: 0x0800202f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amoadd.w", mask: 0xf800707f, value: 0x0000202f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amoxor.w", mask: 0xf800707f, value: 0x2000202f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amoand.w", mask: 0xf800707f, value: 0x6000202f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amoor.w", mask: 0xf800707f, value: 0x4000202f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amomin.w", mask: 0xf800707f, value: 0x8000202f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amomax.w", mask: 0xf800707f, value: 0xa000202f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amominu.w", mask: 0xf800707f, value: 0xc000202f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amomaxu.w", mask: 0xf800707f, value: 0xe000202f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },

    // rv64_a
    Encoding { name: "lr.d", mask: 0xf9f0707f, value: 0x1000302f, fields: &["rd", "rs1", /* "aq", "rl" */] },
    Encoding { name: "sc.d", mask: 0xf800707f, value: 0x1800302f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amoswap.d", mask: 0xf800707f, value: 0x0800302f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amoadd.d", mask: 0xf800707f, value: 0x0000302f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amoxor.d", mask: 0xf800707f, value: 0x2000302f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amoand.d", mask: 0xf800707f, value: 0x6000302f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amoor.d", mask: 0xf800707f, value: 0x4000302f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amomin.d", mask: 0xf800707f, value: 0x8000302f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amomax.d", mask: 0xf800707f, value: 0xa000302f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amominu.d", mask: 0xf800707f, value: 0xc000302f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },
    Encoding { name: "amomaxu.d", mask: 0xf800707f, value: 0xe000302f, fields: &["rd", "rs1", "rs2", /* "aq", "rl" */] },

    // rv_zifencei
    Encoding { name: "fence.i", mask: 0x0000707f, value: 0x0000100f, fields: &["rd", "rs1", "imm12"] },

    // rv_zicsr
    Encoding { name: "csrrw", mask: 0x0000707f, value: 0x00001073, fields: &["rd", "rs1", "csr"] },
    Encoding { name: "csrrs", mask: 0x0000707f, value: 0x00002073, fields: &["rd", "rs1", "csr"] },
    Encoding { name: "csrrc", mask: 0x0000707f, value: 0x00003073, fields: &["rd", "rs1", "csr"] },
    Encoding { name: "csrrwi", mask: 0x0000707f, value: 0x00005073, fields: &["rd", "csr", "zimm"] },
    Encoding { name: "csrrsi", mask: 0x0000707f, value: 0x00006073, fields: &["rd", "csr", "zimm"] },
    Encoding { name: "csrrci", mask: 0x0000707f, value: 0x00007073, fields: &["rd", "csr", "zimm"] },

    // rv_system
    Encoding { name: "mret", mask: 0xffffffff, value: 0x30200073, fields: &[] },
    // Encoding { name: "dret", mask: 0xffffffff, value: 0x7b200073, fields: &[] },
    Encoding { name: "wfi", mask: 0xffffffff, value: 0x10500073, fields: &[] },

    // rv_s
    Encoding { name: "sfence.vma", mask: 0xfe007fff, value: 0x12000073, fields: &["rs1", "rs2"] },
    Encoding { name: "sret", mask: 0xffffffff, value: 0x10200073, fields: &[] },
];

#[rustfmt::skip]
static ENCODINGS_16_DATA: &[Encoding] = &[
    Encoding { name: "c.unimp", mask: 0xffff, value: 0x0000, fields: &[] },

    // rv_c
    Encoding { name: "c.addi4spn", mask: 0xe003, value: 0x0000, fields: &["rd_p", "c_nzuimm10"] },
    Encoding { name: "c.lw", mask: 0xe003, value: 0x4000, fields: &["rd_p", "rs1_p", "c_uimm7hilo"] },
    Encoding { name: "c.sw", mask: 0xe003, value: 0xc000, fields: &["rs1_p", "rs2_p", "c_uimm7hilo"] },
    Encoding { name: "c.nop", mask: 0xef83, value: 0x0001, fields: &["c_nzimm6hilo"] },
    Encoding { name: "c.addi", mask: 0xe003, value: 0x0001, fields: &["rd_rs1_n0", "c_nzimm6hilo"] },
    Encoding { name: "c.li", mask: 0xe003, value: 0x4001, fields: &["rd", "c_imm6hilo"] },
    Encoding { name: "c.addi16sp", mask: 0xef83, value: 0x6101, fields: &["c_nzimm10hilo"] },
    Encoding { name: "c.lui", mask: 0xe003, value: 0x6001, fields: &["rd_n2", "c_nzimm18hilo"] },
    Encoding { name: "c.andi", mask: 0xec03, value: 0x8801, fields: &["rd_rs1_p", "c_imm6hilo"] },
    Encoding { name: "c.sub", mask: 0xfc63, value: 0x8c01, fields: &["rd_rs1_p", "rs2_p"] },
    Encoding { name: "c.xor", mask: 0xfc63, value: 0x8c21, fields: &["rd_rs1_p", "rs2_p"] },
    Encoding { name: "c.or", mask: 0xfc63, value: 0x8c41, fields: &["rd_rs1_p", "rs2_p"] },
    Encoding { name: "c.and", mask: 0xfc63, value: 0x8c61, fields: &["rd_rs1_p", "rs2_p"] },
    Encoding { name: "c.j", mask: 0xe003, value: 0xa001, fields: &["c_imm12"] },
    Encoding { name: "c.beqz", mask: 0xe003, value: 0xc001, fields: &["rs1_p", "c_bimm9hilo"] },
    Encoding { name: "c.bnez", mask: 0xe003, value: 0xe001, fields: &["rs1_p", "c_bimm9hilo"] },
    Encoding { name: "c.lwsp", mask: 0xe003, value: 0x4002, fields: &["rd_n0", "c_uimm8sphilo"] },
    Encoding { name: "c.jr", mask: 0xf07f, value: 0x8002, fields: &["rs1_n0"] },
    Encoding { name: "c.mv", mask: 0xf003, value: 0x8002, fields: &["rd", "c_rs2_n0"] },
    Encoding { name: "c.ebreak", mask: 0xffff, value: 0x9002, fields: &[] },
    Encoding { name: "c.jalr", mask: 0xf07f, value: 0x9002, fields: &["c_rs1_n0"] },
    Encoding { name: "c.add", mask: 0xf003, value: 0x9002, fields: &["rd_rs1", "c_rs2_n0"] },
    Encoding { name: "c.swsp", mask: 0xe003, value: 0xc002, fields: &["c_rs2", "c_uimm8sp_s"] },

    // rv64_c
    Encoding { name: "c.ld", mask: 0xe003, value: 0x6000, fields: &["rd_p", "rs1_p", "c_uimm8hilo"] },
    Encoding { name: "c.sd", mask: 0xe003, value: 0xe000, fields: &["rs1_p", "rs2_p", "c_uimm8hilo"] },
    Encoding { name: "c.addiw", mask: 0xe003, value: 0x2001, fields: &["rd_rs1", "c_imm6hilo"] },
    Encoding { name: "c.srli", mask: 0xec03, value: 0x8001, fields: &["rd_rs1_p", "c_nzuimm6hilo"] },
    Encoding { name: "c.srai", mask: 0xec03, value: 0x8401, fields: &["rd_rs1_p", "c_nzuimm6hilo"] },
    Encoding { name: "c.subw", mask: 0xfc63, value: 0x9c01, fields: &["rd_rs1_p", "rs2_p"] },
    Encoding { name: "c.addw", mask: 0xfc63, value: 0x9c21, fields: &["rd_rs1_p", "rs2_p"] },
    Encoding { name: "c.slli", mask: 0xe003, value: 0x0002, fields: &["rd_rs1_n0", "c_nzuimm6hilo"] },
    Encoding { name: "c.ldsp", mask: 0xe003, value: 0x6002, fields: &["rd_n0", "c_uimm9sphilo"] },
    Encoding { name: "c.sdsp", mask: 0xe003, value: 0xe002, fields: &["c_rs2", "c_uimm9sp_s"] },
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

    ("csr", Field { format: format_value, decode: |v| uf(v,20,12) }),
    ("zimm", Field { format: format_value, decode: |v| uf(v,15,5) }),

    ("rd_n0", Field { format: format_reg, decode: |v| uf(v,7,5) }),
    ("rd_n2", Field { format: format_reg, decode: |v| uf(v,7,5) }),
    ("rd_rs1", Field { format: format_reg, decode: |v| uf(v,7,5) }),
    ("rd_rs1_n0", Field { format: format_reg, decode: |v| uf(v,7,5) }),
    ("rd_p", Field { format: format_reg, decode: |v| 8 + uf(v,2,3) }),
    ("rd_rs1_p", Field { format: format_reg, decode: |v| 8 + uf(v,7,3) }),

    ("rs1_n0", Field { format: format_reg, decode: |v| uf(v,7,5) }),
    ("rs1_p", Field { format: format_reg, decode: |v| 8 + uf(v,7,3) }),
    ("c_rs1_n0", Field { format: format_reg, decode: |v| uf(v,7,5) }),

    ("rs2_p", Field { format: format_reg, decode: |v| 8 + uf(v,2,3) }),
    ("c_rs2", Field { format: format_reg, decode: |v| uf(v,2,5) }),
    ("c_rs2_n0", Field { format: format_reg, decode: |v| uf(v,2,5) }),

    ("c_bimm9hilo", Field { format: format_value, decode: |v| (uf(v,3,2) << 1) + (uf(v,10,2) << 3) + (uf(v,2,1) << 5) + (uf(v,5,2) << 6) + (sf(v,12, 1) << 8) }),
    ("c_imm12", Field { format: format_value, decode: |v| (uf(v,3,3) << 1) + (uf(v,11,1) << 4) + (uf(v,2,1) << 5) + (uf(v,7,1) << 6) + (uf(v,6,1) << 7) + (uf(v,9,2) << 8) + (uf(v,8,1) << 10) + (sf(v,12,1) << 11) }),
    ("c_imm6hilo", Field { format: format_value, decode: |v| uf(v,2,5) + (sf(v,12,1) << 5) }),
    ("c_nzimm10hilo", Field { format: format_value, decode: |v| (uf(v,6,1) << 4) + (uf(v,2,1) << 5) + (uf(v,5,1) << 6) + (uf(v,3,2) << 7) + (sf(v,12,1) << 9) }),
    ("c_nzimm18hilo", Field { format: format_value, decode: |v| (uf(v,2,5) + (sf(v,12,1) << 5)) << 12 }),
    ("c_nzimm6hilo", Field { format: format_value, decode: |v| uf(v,2,5) + (sf(v,12,1) << 5) }),
    ("c_nzuimm6hilo", Field { format: format_value, decode: |v| uf(v,2,5) + (uf(v,12,1) << 5) }),
    ("c_nzuimm10", Field { format: format_value, decode: |v| (uf(v,6,1) << 2) + (uf(v,5,1) << 3) + (uf(v,11,2) << 4) + (uf(v,7,4) << 6) }),
    ("c_uimm7hilo", Field { format: format_value, decode: |v| (uf(v,6,1) << 2) + (uf(v,10,3) << 3) + (uf(v,5,1) << 6) }),
    ("c_uimm8hilo", Field { format: format_value, decode: |v| (uf(v,10,3) << 3) + (uf(v,5,2) << 6) }),
    ("c_uimm8sp_s", Field { format: format_value, decode: |v| (uf(v,9,4) << 2) + (uf(v,7,2) << 6) }),
    ("c_uimm9sp_s", Field { format: format_value, decode: |v| (uf(v,10,3) << 3) + (uf(v,7,3) << 6) }),
    ("c_uimm8sphilo", Field { format: format_value, decode: |v| (uf(v,4,3) << 2) + (uf(v,12,1) << 5) + (uf(v,2,2) << 6) }),
    ("c_uimm9sphilo", Field { format: format_value, decode: |v| (uf(v,5,2) << 3) + (uf(v,12,1) << 5) + (uf(v,2,3) << 6) }),
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
    static ref ENCODINGS_16: HashMap<u32, Vec<Encoding>> = {
        let mut res: HashMap<u32, Vec<Encoding>> = HashMap::new();

        for enc in ENCODINGS_16_DATA {
            assert!((enc.mask >> 13 << 2) | (enc.mask & 0x3) == 0b11111);
            res.entry((enc.value >> 13 << 2) | (enc.value & 0x3))
                .or_default()
                .push(*enc);
        }

        res
    };
    static ref FIELDS: HashMap<&'static str, Field> = FIELDS_DATA.iter().copied().collect();
}

pub fn decode(insn: u32) -> Option<Encoding> {
    if insn & 0b11 == 0b11 {
        // 32b insn
        ENCODINGS_32.get(&(insn & 0x7f)).and_then(|encs| {
            encs.iter()
                .find(|enc| (insn & enc.mask) == enc.value)
                .copied()
        })
    } else {
        ENCODINGS_16
            .get(&((insn >> 13 << 2) | (insn & 0x3)))
            .and_then(|encs| {
                encs.iter()
                    .find(|enc| (insn & enc.mask) == enc.value)
                    .copied()
            })
    }
}

pub fn field(name: &str) -> Field {
    *FIELDS.get(name).expect(name)
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
