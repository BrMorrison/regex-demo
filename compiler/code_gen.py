from . import syntax
from .instruction import *
from functools import singledispatch

# The wildcard and die operations are encoded as special cases of the `Compare` instruction that
# use an invalid character (0xFF).
_wildcard_instruction = InvCompare("%255", "%255")
_die_instruction = Compare("%255", "%255")

def compile(val: syntax.Construction) -> list[Instruction]:
    code, _ = compile_helper(val, 0)

    # Stick a match at the end to represent a successful match.
    code.append(Match())
    return code

@singledispatch
def compile_helper(val, pc:int) -> tuple[list[Instruction], int]:
    raise AssertionError(f"Unexpected type for val {val.type}")

@compile_helper.register
def _(val: syntax.Literal, pc: int) -> tuple[list[Instruction], int]:
    escaped = escape_encode(val.val)
    return ([Compare(escaped, escaped)], pc+1)

@compile_helper.register
def _(val: syntax.Group, pc: int) -> tuple[list[Instruction], int]:
    save_index = val.expression_index*2
    exp_code, pc2 = compile_helper(val.expression, pc+1)
    code = [Save(save_index)] + exp_code + [Save(save_index+1)]
    return (code, pc2+1)

@compile_helper.register
def _(_: syntax.WildCard, pc: int) -> tuple[list[Instruction], int]:
    return ([_wildcard_instruction], pc+1)

@compile_helper.register
def _(val: syntax.CharSet, pc: int) -> tuple[list[Instruction], int]:
    # Handle single characters and ranges separately since they don't actually need options.
    inst = InvCompare if val.inverse else Compare
    if val._is_single_char():
        escaped = escape_encode(val.chars[0])
        return ([inst(escaped, escaped)], pc+1)
    elif val._is_single_range():
        c_min, c_max = val.ranges[0]
        return ([inst(escape_encode(c_min), escape_encode(c_max))], pc+1)
    
    # More complex character sets require a series of commands
    """
    ---- Normal Comparison ----
        OptCompare for single characters <dest=L1>
        OptCompare for character ranges <dest=L1>
    L0: Die
    L1: Consume
    L2:
    ---- Inverse Comparison ----
        OptCompare for single characters <dest=L1>
        OptCompare for character ranges <dest=L1>
    L0: Consume
        Jump L2
    L1: Die
    L2:
    """
    code = []
    l0 = pc + len(val.chars) + len(val.ranges)

    if not val.inverse:
        l1 = l0 + 1
        l2 = l0 + 2
        code_postfix = [_die_instruction, _wildcard_instruction]
    else:
        l1 = l0 + 2
        l2 = l0 + 3
        code_postfix = [_wildcard_instruction, Jump(l2), _die_instruction]

    for c in val.chars:
        code.append(OptCompare(escape_encode(c), escape_encode(c), l1))
    for c_min, c_max in val.ranges:
        code.append(OptCompare(escape_encode(c_min), escape_encode(c_max), l1))
    code += code_postfix

    return (code, l2)

@compile_helper.register
def _(val: syntax.Sequence, pc: int) -> tuple[list[Instruction], int]:
    code = []
    for seq_val in val.val:
        tmp_code, pc = compile_helper(seq_val, pc)
        code += tmp_code
    return (code, pc)

@compile_helper.register
def _(val: syntax.Alternatives, pc: int) -> tuple[list[Instruction], int]:
    """
        Split L1, L2
    L1: code for alt1
        Jump L3:
    L2: code for alt2
    L3:
    """
    l1 = pc+1
    code1, pc1 = compile_helper(val.alt1, l1)
    l2 = pc1+1
    code2, l3 = compile_helper(val.alt2, l2)
    return ([Split(l1, l2)] + code1 + [Jump(l3)] + code2, l3)

@compile_helper.register
def _(val: syntax.Option, pc: int) -> tuple[list[Instruction], int]:
    """
        Split L1, L2
    L1: code for val
    L2:
    """
    # NOTE: We could do a strength-reduction optimization to optChar or OptRange if the option
    # is on a simple character or range.
    l1 = pc+1
    code, l2 = compile_helper(val.val, l1)
    return ([Split(l1, l2)] + code, l2)

@compile_helper.register
def _(val: syntax.Some, pc: int) -> tuple[list[Instruction], int]:
    """
    L1: code for val
        Split L1, L3
    L3:
    """
    l1 = pc
    code, pc1 = compile_helper(val.val, l1)
    l3 = pc1+1
    return (code + [Split(l1, l3)], l3)

@compile_helper.register
def _(val: syntax.Any, pc: int) -> tuple[list[Instruction], int]:
    """
    L1: Split L2, L3
    L2: code for val
        Jump L1
    L3:
    """
    l1 = pc
    l2 = pc+1
    code, pc1 = compile_helper(val.val, l2)
    l3 = pc1+1
    return ([Split(l2, l3)] + code + [Jump(l1)], l3)

def escape_encode(c: str) -> str:
    '''
    Escapes characters that need escaping in code generation, like '%', ',', and spaces, which have
    special meaning in the assembly.
    '''
    assert len(c) == 1, "Can only escape single characters."
    print_val = c
    if c.isspace() or c in ['%', ',']:
        print_val = f"%{int.from_bytes(c.encode('utf-8'))}"
    return print_val
