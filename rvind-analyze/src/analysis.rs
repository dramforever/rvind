use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    fmt,
    hash::Hash,
    ops::Range,
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Reg(u8);

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &crate::riscv::REG_NAMES[self.0 as usize])
    }
}

impl Reg {
    fn from(value: i64) -> Option<Self> {
        assert!((0..32).contains(&value));
        (value > 0).then_some(Reg(value as u8))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Nop,
    Unreachable,
    Tail,
    Const { dest: Reg, value: i64 },
    Add { dest: Reg, base: Reg, offset: i64 },
    Load { dest: Reg, base: Reg, offset: i64 },
    Store { val: Reg, base: Reg, offset: i64 },
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Operation::*;

        match self {
            Nop => write!(f, "nop"),
            Unreachable => write!(f, "unreachable!"),
            Tail => write!(f, "tail"),
            Const { dest, value } => write!(f, "const {dest} <- {value}"),
            Add { dest, base, offset } => write!(f, "add {dest} <- {offset} + {base}"),
            Load { dest, base, offset } => write!(f, "load {dest} <- {offset}({base})"),
            Store { val, base, offset } => write!(f, "store {val} -> {offset}({base})"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InsnAnalysis {
    operation: Operation,
    clobbers: Vec<Reg>,
    successors: Vec<i64>,
}

impl fmt::Display for InsnAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, clobber ", self.operation)?;
        let mut fl = f.debug_list();
        for c in &self.clobbers {
            fl.entry(&format_args!("{c}"));
        }
        fl.finish()?;
        write!(f, ", -> {:+?}", self.successors)
    }
}

const UNIMP: InsnAnalysis = InsnAnalysis {
    operation: Operation::Unreachable,
    clobbers: Vec::new(),
    successors: Vec::new(),
};

pub fn analyze_insn(pc: i64, range: &Range<i64>, insn: u32) -> InsnAnalysis {
    use Operation::*;

    let enc = if let Some(enc) = crate::riscv::decode(insn) {
        enc
    } else {
        return UNIMP;
    };

    let fields: HashMap<&'static str, i64> = enc
        .fields
        .iter()
        .map(|&f| (f, (crate::riscv::field(f).decode)(insn)))
        .collect();

    let next = if insn & 0b11 == 0b11 { 4 } else { 2 };

    match enc.name {
        "addi" => {
            let operation = if let Some(rd) = Reg::from(fields["rd"]) {
                if let Some(rs1) = Reg::from(fields["rs1"]) {
                    Add {
                        dest: rd,
                        base: rs1,
                        offset: fields["imm12"],
                    }
                } else {
                    Const {
                        dest: rd,
                        value: fields["imm12"],
                    }
                }
            } else {
                Nop
            };

            InsnAnalysis {
                operation,
                clobbers: Vec::new(),
                successors: vec![next],
            }
        }

        "ld" => {
            if let (Some(rd), Some(rs1)) = (Reg::from(fields["rd"]), Reg::from(fields["rs1"])) {
                InsnAnalysis {
                    operation: Load {
                        dest: rd,
                        base: rs1,
                        offset: fields["imm12"],
                    },
                    clobbers: Vec::new(),
                    successors: vec![next],
                }
            } else {
                InsnAnalysis {
                    operation: Nop,
                    clobbers: Reg::from(fields["rd"]).into_iter().collect(),
                    successors: vec![next],
                }
            }
        }

        "sd" => {
            let operation = if let (Some(rs1), Some(rs2)) =
                (Reg::from(fields["rs1"]), Reg::from(fields["rs2"]))
            {
                Store {
                    val: rs2,
                    base: rs1,
                    offset: fields["imm12hilo"],
                }
            } else {
                Nop
            };

            InsnAnalysis {
                operation,
                clobbers: Vec::new(),
                successors: vec![next],
            }
        }

        "jal" => {
            if let Some(rd) = Reg::from(fields["rd"]) {
                InsnAnalysis {
                    operation: Nop,
                    clobbers: vec![rd], // FIXME: ABI clobber
                    successors: vec![next],
                }
            } else {
                let off = fields["jimm20"];

                if range.contains(&pc.wrapping_add(off)) {
                    InsnAnalysis {
                        operation: Nop,
                        clobbers: Vec::new(),
                        successors: vec![off],
                    }
                } else {
                    InsnAnalysis {
                        operation: Tail,
                        clobbers: Vec::new(),
                        successors: Vec::new(),
                    }
                }
            }
        }

        "jalr" => {
            if let Some(rd) = Reg::from(fields["rd"]) {
                InsnAnalysis {
                    operation: Nop,
                    clobbers: vec![rd], // FIXME: ABI clobber
                    successors: vec![next],
                }
            } else {
                InsnAnalysis {
                    operation: Tail,
                    clobbers: Vec::new(),
                    successors: Vec::new(),
                }
            }
        }

        "beq" | "bne" | "blt" | "bge" | "bltu" | "bgeu" => InsnAnalysis {
            operation: Nop,
            clobbers: Vec::new(),
            successors: vec![next, fields["bimm12hilo"]],
        },

        // FIXME: Maybe there's a better way...
        "auipc" | "lb" | "lh" | "lw" | "lbu" | "lhu" | "lwu" | "slti" | "sltiu" | "xori"
        | "ori" | "andi" | "add" | "sub" | "sll" | "slt" | "sltu" | "xor" | "srl" | "sra"
        | "or" | "and" | "slli" | "srli" | "srai" | "addiw" | "slliw" | "srliw" | "sraiw"
        | "addw" | "subw" | "sllw" | "srlw" | "sraw" => InsnAnalysis {
            operation: Nop,
            clobbers: Reg::from(fields["rd"]).into_iter().collect(),
            successors: vec![next],
        },

        "sb" | "sh" | "sw" => InsnAnalysis {
            operation: Nop,
            clobbers: Vec::new(),
            successors: vec![next],
        },

        _ => {
            eprintln!("Unhandled instruction {}", enc.name);
            UNIMP
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownValue {
    Abs(i64),
    OrigSp(i64),
    OrigFp,
    OrigRa,
}

impl KnownValue {
    fn add(self, offset: i64) -> Option<Self> {
        use KnownValue::*;

        match self {
            Abs(val) => Some(Abs(val.wrapping_add(offset))),
            OrigSp(val) => Some(OrigSp(val.wrapping_add(offset))),
            OrigFp => None,
            OrigRa => None,
        }
    }
}

impl fmt::Display for KnownValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use KnownValue::*;

        match self {
            Abs(val) => write!(f, "{val:#x}"),
            OrigSp(val) => write!(f, "_sp + {val}"),
            OrigFp => write!(f, "_fp"),
            OrigRa => write!(f, "_ra"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AbstractState {
    regs: BTreeMap<Reg, KnownValue>,
    stack: BTreeMap<i64, KnownValue>,
}

fn merge_map<K: Ord, V: Eq>(current: &mut BTreeMap<K, V>, other: &BTreeMap<K, V>) -> bool {
    let mut changed = false;

    current.retain(|k, v| {
        let good = if let Some(other_v) = other.get(k) {
            v == other_v
        } else {
            false
        };

        changed = changed || !good;

        good
    });

    changed
}

impl AbstractState {
    fn execute_operation(&mut self, op: Operation) {
        use KnownValue::*;
        use Operation::*;

        match op {
            Nop => {}
            Unreachable => {}
            Tail => {}
            Const { dest, value } => {
                self.regs.insert(dest, Abs(value));
            }
            Add { dest, base, offset } => {
                if let Some(val) = self.regs.get(&base) {
                    if let Some(new_val) = val.add(offset) {
                        self.regs.insert(dest, new_val);
                    } else {
                        self.regs.remove(&dest);
                    }
                }
            }
            Load { dest, base, offset } => {
                if let Some(OrigSp(sp_off)) = self.regs.get(&base).and_then(|v| v.add(offset)) {
                    if let Some(val) = self.stack.get(&sp_off) {
                        self.regs.insert(dest, *val);
                    } else {
                        self.regs.remove(&dest);
                    }
                } else {
                    self.regs.remove(&dest);
                }
            }

            Store { val, base, offset } => {
                if let Some(OrigSp(sp_off)) = self.regs.get(&base).and_then(|v| v.add(offset)) {
                    if let Some(val) = self.regs.get(&val) {
                        self.stack.insert(sp_off, *val);
                    } else {
                        self.stack.remove(&sp_off);
                    }
                }
            }
        }
    }

    fn execute(&mut self, analysis: &InsnAnalysis) {
        for c in &analysis.clobbers {
            self.regs.remove(c);
        }
        self.execute_operation(analysis.operation);
    }

    fn merge(&mut self, other: &Self) -> bool {
        merge_map(&mut self.regs, &other.regs) || merge_map(&mut self.stack, &other.stack)
    }
}

impl fmt::Display for AbstractState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fl = f.debug_list();
        for (reg, val) in &self.regs {
            fl.entry(&format_args!("{reg} = {val}"));
        }

        for (off, val) in &self.stack {
            fl.entry(&format_args!("{off}(_sp) = {val}"));
        }
        fl.finish()
    }
}

pub fn analyze(addr: i64, bytes: &[u8]) -> HashMap<i64, AbstractState> {
    let range = addr..addr + (bytes.len() as i64);

    let mut res: HashMap<i64, AbstractState> = HashMap::new();
    let mut queue: VecDeque<i64> = VecDeque::new();
    queue.push_back(addr);
    res.insert(
        addr,
        AbstractState {
            regs: [
                (Reg::from(1).unwrap(), KnownValue::OrigRa),
                (Reg::from(2).unwrap(), KnownValue::OrigSp(0)),
                (Reg::from(8).unwrap(), KnownValue::OrigFp),
            ]
            .into(),
            stack: [].into(),
        },
    );

    while let Some(pc) = queue.pop_front() {
        let mut state = res[&pc].clone();

        if let Some(first) = bytes.first() {
            let off = (pc - addr) as usize;
            let insn = if first & 0b11 == 0b11 {
                u32::from_le_bytes(bytes[off..][..4].try_into().unwrap())
            } else {
                u16::from_le_bytes(bytes[off..][..2].try_into().unwrap()) as u32
            };

            let analysis = analyze_insn(pc, &range, insn);

            state.execute(&analysis);

            for succ in &analysis.successors {
                if let Some(s) = res.get_mut(&(pc + succ)) {
                    if s.merge(&state) {
                        queue.push_back(pc + succ);
                    }
                } else {
                    res.insert(pc + succ, state.clone());
                    queue.push_back(pc + succ);
                }
            }
        }
    }

    res
}
