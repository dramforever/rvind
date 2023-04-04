import sys
import json

from typing import *

FIELD_NAMES = ['rd','rs1','rs2','imm20','jimm20','imm12','imm12hilo','bimm12hilo','shamtd','shamtw','fm','pred','succ']
FIELD_ORDER = { n: i for i, n in enumerate(FIELD_NAMES) }

def normalize_part(part: str) -> str:
    if part.endswith('hi'): return f'{part}lo'
    if part.endswith('lo'): return None
    return part

def fixed_part(part: str) -> str:
    br, val = part.split('=')
    hi, lo = br.split('..')
    val, hi, lo = int(val, 0), int(hi), int(lo)
    return (val << lo, (1 << (hi + 1)) - (1 << lo))

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

        print(f'Encoding {{ name: "{name}", mask: 0x{mask:08x}, value: 0x{fixed:08x}, fields: &{json.dumps(sorted(fields, key=lambda n: FIELD_ORDER[n]))} }},')

if __name__ == '__main__':
    match sys.argv:
        case [script]:
            process(sys.stdin)
        case [script, name]:
            with open(name) as f:
                process(f)
        case [script, *_]:
            print(f'Usage: {script} [<file>]')
