from . import instruction as inst

from functools import singledispatch
from enum import IntEnum

class Opcode(IntEnum):
    Jump = 0b000
    Split = 0b001
    Compare = 0b010
    Branch = 0b011
    Save = 0b100
    Match = 0b111

_opcode_shift = 29
_save_index_shift = 16 # This one can be wherever
_inverted_shift = 28
_dest_shift = 16
_dest2_shift = 2
_char_min_shift = 8
_char_max_shift = 0

def char_to_int(c: str) -> int:
    if len(c) > 1:
        assert c[0] == '%'
        return int(c[1:])
    else:
        return int.from_bytes(c.encode('utf-8'))

# Returns a 32-bit number
@singledispatch
def assemble(val) -> int:
    raise AssertionError(f"Unexpected type for val {val.type}")

@assemble.register
def _(val: inst.Match) -> int:
    asm = (Opcode.Match.value << _opcode_shift)
    return asm & 0xFFFF_FFFF

@assemble.register
def _(val: inst.Save) -> int:
    asm = (Opcode.Save.value << _opcode_shift) | (val.index << _save_index_shift)
    return asm & 0xFFFF_FFFF

@assemble.register
def _(val: inst.Jump) -> int:
    asm = (Opcode.Jump.value << _opcode_shift) | (val.dest << _dest_shift)
    return asm & 0xFFFF_FFFF

@assemble.register
def _(val: inst.Split) -> int:
    asm = (Opcode.Split.value << _opcode_shift) | (val.dest1 << _dest_shift) | (val.dest2 << _dest2_shift)
    return asm & 0xFFFF_FFFF

@assemble.register
def _(val: inst.Compare) -> int:
    asm = (Opcode.Compare.value << _opcode_shift) | (int(val.inverted) << _inverted_shift) \
        | (char_to_int(val.escaped_char1) << _char_min_shift) \
        | (char_to_int(val.escaped_char2) << _char_max_shift)
    return asm & 0xFFFF_FFFF

@assemble.register
def _(val: inst.Branch) -> int:
    asm = (Opcode.Branch.value << _opcode_shift) | (val.dest << _dest_shift) \
        | (char_to_int(val.escaped_char1) << _char_min_shift) \
        | (char_to_int(val.escaped_char2) << _char_max_shift)
    return asm & 0xFFFF_FFFF
