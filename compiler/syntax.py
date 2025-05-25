from dataclasses import dataclass

# The wildcard and die operations are encoded as special cases of the `Compare` instruction that
# use an invalid character (0xFF).
_wildcard_instruction = "InvCompare %255 %255\n"
_die_instruction = "Compare %255 %255\n"

"""
Instructions:

- Match
- Jump <dest>
- Split <dest1> <dest2>
- Compare <char1> <char2>
- InvCompare <char1> <char2>
- OptCompare <char1> <char2> <dest>

Not currently supported:
- Save <index>
"""

#region types

class Construction:
    '''
    Base type for all regular expression grammar constructions.
    '''
    def compile(self, pc: int=0) -> tuple[str, int]:
        return ('', pc)

@dataclass
class Literal(Construction):
    '''
    A single character literal to match.
    '''
    val: str

    def compile(self, pc: int=0) -> tuple[str, int]:
        escaped = escape_encode(self.val)
        return (f"Compare {escaped} {escaped} \n", pc+1)

@dataclass
class Group(Construction):
    '''
    A subexpression that can be matched and have it's location extracted afterwards.
    Currently just supported for the top-level expression.
    '''
    expression_index: int
    expression: Construction

    def compile(self, pc: int=0) -> tuple[str, int]:
        save_index = self.expression_index*2
        exp_code, pc2 = self.expression.compile(pc+1)
        code = f"Save {save_index}\n{exp_code}Save {save_index+1}\n"
        return (code, pc2+1)

@dataclass
class WildCard(Construction):
    '''
    Matches any single character, ex: `.`
    '''
    def compile(self, pc: int=0) -> tuple[str, int]:
        return (_wildcard_instruction, pc+1)

@dataclass
class CharSet(Construction):
    '''
    Matches any single character in a set, ex: `[a-z0-9]`
    '''
    ranges: list[tuple[str, str]]
    chars: list[str]
    inverse: bool

    def _is_single_char(self) -> bool:
        return len(self.chars) == 1 and len(self.ranges) == 0

    def _is_single_range(self) -> bool:
        return len(self.chars) == 0 and len(self.ranges) == 1

    def compile(self, pc: int=0) -> tuple[str, int]:
        # Handle single characters and ranges separately since they don't actually need options.
        command = "InvCompare" if self.inverse else "Compare"
        if self._is_single_char():
            escaped = escape_encode(self.chars[0])
            return (f"{command} {escaped}, {escaped}\n", pc+1)
        elif self._is_single_range():
            c_min, c_max = self.ranges[0]
            return (f"{command} {escape_encode(c_min)} {escape_encode(c_max)}\n", pc+1)
        
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
        code = ''
        l0 = pc + len(self.chars) + len(self.ranges)

        if not self.inverse:
            l1 = l0 + 1
            l2 = l0 + 2
            code_postfix = _die_instruction + _wildcard_instruction
        else:
            l1 = l0 + 2
            l2 = l0 + 3
            code_postfix = _wildcard_instruction + f"Jump {l2}\n" + _die_instruction

        for c in self.chars:
            escaped = escape_encode(c)
            code += f"OptCompare {escaped} {escaped} {l1}\n"
        for c_min, c_max in self.ranges:
            code += f"OptCompare {escape_encode(c_min)} {escape_encode(c_max)} {l1}\n"
        code += code_postfix

        return (code, l2)

@dataclass
class Sequence(Construction):
    '''
    Matches a sequence of sub-expressions, ex: `abc`
    '''
    val: list[Construction]

    def compile(self, pc: int=0) -> tuple[str, int]:
        code = ''
        for val in self.val:
            tmp_code, pc = val.compile(pc)
            code += tmp_code
        return (code, pc)

@dataclass
class Alternatives(Construction):
    '''
    Matches one of multiple alternative sub-expressions, ex: `ab|cd`
    '''
    alt1: Construction
    alt2: Construction

    def compile(self, pc: int=0) -> tuple[str, int]:
        """
            Split L1, L2
        L1: code for alt1
            Jump L3:
        L2: code for alt2
        L3:
        """
        l1 = pc+1
        code1, pc1 = self.alt1.compile(l1)
        l2 = pc1+1
        code2, l3 = self.alt2.compile(l2)
        return (f"Split {l1} {l2}\n" + code1 + f"Jump {l3}\n" + code2, l3)

@dataclass
class Option(Construction):
    '''
    Matches zero or one occurrences, ex: `a?`
    '''
    val: Construction

    def compile(self, pc: int=0) -> tuple[str, int]:
        """
            Split L1, L2
        L1: code for val
        L2:
        """
        # NOTE: We could do a strength-reduction optimization to optChar or OptRange if the option
        # is on a simple character or range.
        l1 = pc+1
        code, l2 = self.val.compile(l1)
        return (f"Split {l1} {l2}\n" + code, l2)

@dataclass
class Some(Construction):
    '''
    Matches one or more occurrences, ex: `a+`
    '''
    val: Construction

    def compile(self, pc: int=0) -> tuple[str, int]:
        """
        L1: code for val
            Split L1, L3
        L3:
        """
        l1 = pc
        code, pc1 = self.val.compile(l1)
        l3 = pc1+1
        return (code + f"Split {l1} {l3}\n", l3)

@dataclass
class Any(Construction):
    '''
    Matches zero or more occurrences, ex: `a*`
    '''
    val: Construction

    def compile(self, pc: int=0) -> tuple[str, int]:
        """
        L1: Split L2, L3
        L2: code for val
            Jump L1
        L3:
        """
        l1 = pc
        l2 = pc+1
        code, pc1 = self.val.compile(l2)
        l3 = pc1+1
        return (f"Split {l2} {l3}\n" + code + f"Jump {l1}\n", l3)

#endregion

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
