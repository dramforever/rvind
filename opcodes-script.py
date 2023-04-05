import sys
import json

from typing import *

FIELD_NAMES = [
    'rd', 'rd_p', 'rd_n0', 'rd_n2', 'rd_rs1', 'rd_rs1_p', 'rd_rs1_n0',
    'rs1', 'rs1_n0', 'rs1_p', 'c_rs1_n0',
    'rs2', 'rs2_p', 'c_rs2', 'c_rs2_n0',
    'imm20', 'jimm20', 'imm12', 'csr', 'imm12hilo', 'bimm12hilo',
    'c_bimm9hilo', 'c_imm12', 'c_imm6hilo', 'c_nzimm10hilo', 'c_nzimm18hilo', 'c_nzimm6hilo', 'c_nzuimm6hilo', 'c_nzuimm10', 'c_uimm7hilo', 'c_uimm8hilo', 'c_uimm8sp_s', 'c_uimm9sp_s', 'c_uimm8sphilo', 'c_uimm9sphilo',
    'shamtd', 'shamtw',
    'fm', 'pred', 'succ',
    'aq', 'rl',
    'zimm'
]
FIELD_ORDER = { n: i for i, n in enumerate(FIELD_NAMES) }

def normalize_part(part: str) -> str:
    if part.endswith('hi'): return f'{part}lo'
    if part.endswith('lo'): return None
    return part

def fixed_part(part: str) -> str:
    br, val = part.split('=')
    match br.split('..'):
        case [hi, lo]:
            val, hi, lo = int(val, 0), int(hi), int(lo)
            return (val << lo, (1 << (hi + 1)) - (1 << lo))
        case [b]:
            val, b = int(val, 0), int(b)
            return (val << b, 1 << b)

def process(file: TextIO):
    for line in file.readlines():
        if line.startswith('#'): continue # Comment
        line = line.rstrip('\n').split()
        if not line: continue # Empty
        if line[0].startswith('$'): continue # Directive

        name, *parts = line
        parts = [ normalize_part(part) for part in parts ]
        parts = [ p for p in parts if p is not None ]

        fixed, mask, fields = 0, 0, []

        for part in parts:
            if part[0].isdigit():
                pfixed, pmask = fixed_part(part)
                fixed |= pfixed
                mask |= pmask
            else:
                fields.append(part)

        assert mask & 0b11 == 0b11

        if fixed & 0b11 == 0b11:
            print(f'Encoding {{ name: "{name}", mask: 0x{mask:08x}, value: 0x{fixed:08x}, fields: &{json.dumps(sorted(fields, key=lambda n: FIELD_ORDER[n]))} }},')
        else:
            print(f'Encoding {{ name: "{name}", mask: 0x{mask:04x}, value: 0x{fixed:04x}, fields: &{json.dumps(sorted(fields, key=lambda n: FIELD_ORDER[n]))} }},')

if __name__ == '__main__':
    match sys.argv:
        case [script]:
            process(sys.stdin)
        case [script, name]:
            with open(name) as f:
                process(f)
        case [script, *_]:
            print(f'Usage: {script} [<file>]')
