#!/bin/env python3

from dataclasses import dataclass
import sys

_whitespace_chars = ['\n', ' ', '\t', '\r', '\f', '\v']
_alpha_num_ranges = [('0','9'), ('A', 'Z'), ('a', 'z')]
_alpha_num_chars = ['_']
_num_ranges = [('0', '9')]

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
        return (f"char {escape_encode(self.val)}\n", pc+1)

@dataclass
class WildCard(Construction):
    '''
    Matches any single character, ex: `.`
    '''
    def compile(self, pc: int=0) -> tuple[str, int]:
        return (f"consume\n", pc+1)

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

    def _compile_inverse(self, pc) -> tuple[str, int]:
        assert self.inverse == True
        code = ''
        final_pc = pc + len(self.chars) + len(self.ranges) + 1
        for c in self.chars:
            code += f"invChar {escape_encode(c)}\n"
        for c_min, c_max in self.ranges:
            code += f"invRange {escape_encode(c_min)} {escape_encode(c_max)}\n"

        # Inverted character sets end with a wildcard since none of the individual statements
        # consume input.
        return (code + 'consume\n', final_pc)

    def _compile_normal(self, pc) -> tuple[str, int]:
        assert self.inverse == False

        # Handle single characters and ranges separately since they don't actually need options.
        if self._is_single_char():
            return (f"char {escape_encode(self.chars[0])}\n", pc+1)
        elif self._is_single_range():
            c_min, c_max = self.ranges[0]
            return (f"range {escape_encode(c_min)} {escape_encode(c_max)}\n", pc+1)

        code = ''
        final_pc = pc + len(self.chars) + len(self.ranges) + 1
        for c in self.chars:
            code += f"optChar {escape_encode(c)} {final_pc}\n"
        for c_min, c_max in self.ranges:
            code += f"optRange {escape_encode(c_min)} {escape_encode(c_max)} {final_pc}\n"
        return (code + 'die\n', final_pc)

    def compile(self, pc: int=0) -> tuple[str, int]:
        if self.inverse:
            return self._compile_inverse(pc)
        else:
            return self._compile_normal(pc)

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
            split L1, L2
        L1: code for alt1
            jump L3:
        L2: code for alt2
        L3:
        """
        l1 = pc+1
        code1, pc1 = self.alt1.compile(l1)
        l2 = pc1+1
        code2, l3 = self.alt2.compile(l2)
        return (f"split {l1} {l2}\n" + code1 + f"jump {l3}\n" + code2, l3)

@dataclass
class Option(Construction):
    '''
    Matches zero or one occurrences, ex: `a?`
    '''
    val: Construction

    def compile(self, pc: int=0) -> tuple[str, int]:
        """
            split L1, L2
        L1: code for val
        L2:
        """
        # NOTE: We could do a strength-reduction optimization to optChar or OptRange if the option
        # is on a simple character or range.
        l1 = pc+1
        code, l2 = self.val.compile(l1)
        return (f"split {l1} {l2}\n" + code, l2)

@dataclass
class Some(Construction):
    '''
    Matches one or more occurrences, ex: `a+`
    '''
    val: Construction

    def compile(self, pc: int=0) -> tuple[str, int]:
        """
        L1: code for val
            split L1, L3
        L3:
        """
        l1 = pc
        code, pc1 = self.val.compile(l1)
        l3 = pc1+1
        return (code + f"split {l1} {l3}\n", l3)

@dataclass
class Any(Construction):
    '''
    Matches zero or more occurrences, ex: `a*`
    '''
    val: Construction

    def compile(self, pc: int=0) -> tuple[str, int]:
        """
        L1: split L2, L3
        L2: code for val
            jump L1
        L3:
        """
        l1 = pc
        l2 = pc+1
        code, pc1 = self.val.compile(l2)
        l3 = pc1+1
        return (f"split {l2} {l3}\n" + code + f"jump {l1}\n", l3)

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

def find_closing_paren(s: str) -> int | None:
    '''
    Find the closing parenthesis for a string that starts with an opening parenthesis
    '''
    assert s[0] == '('
    depth = 0
    escape: bool = False
    for i, c in enumerate(s):
        # If we hit an escape character last time, then skip this
        if escape:
            escape = False
            continue
        match c:
            case '\\':
                escape = True
            case '(':
                depth += 1
            case ')':
                depth -= 1
                if depth == 0:
                    return i
    return None

def parse_count(s: str) -> tuple[int, int, int]:
    '''
    Parses a count specifier of the form `{n}` or `{min, max}`
    returns a tuple of the min and max counts.
    '''
    assert s[0] == '{'
    end = s.find('}')
    assert end != -1, f"Could not find closing brace in {s}"
    inside = s[1:end]
    nums = inside.split(',')

    if len(nums) == 1:
        count = int(nums[0].strip())
        assert count > 0
        return (count, count, end)
    elif len(nums) == 2:
        min_count = int(nums[0].strip())
        max_count = int(nums[1].strip())
        assert min_count > 0
        assert max_count > min_count
        return (min_count, max_count, end)
    else:
        assert False, f"Invalid count specifier: '{s[:end+1]}'"

def parse_charset(s: str) -> tuple[CharSet, int]:
    assert s[0] == '['
    end = s.find(']')

    # If there's an escape before the closing brace, it doesn't count.
    if end != -1 and s[end-1] == '\\':
        end = s[end:].find(']')
    assert end != -1, f"Could not find closing brace in {s}"

    # See if we're going to invert the character set.
    # Also don't processed the inversion character in the main loop.
    inverted = s[1] == '^'
    inside = s[2:end] if inverted else s[1:end]
    assert len(inside) != 0, f"There must be one or characters inside the set '{s[:end]}'"

    index = 0
    ranges: list[tuple[str, str]] = []
    chars: list[str] = []

    while index < len(inside):
        match inside[index]:
            case '-':
                if len(chars) == 0 or index == len(inside) - 1:
                    chars.append('-')
                elif inside[index+1] != '\\':
                    last = chars.pop()
                    next = inside[index+1]
                    assert last.isalnum() and next.isalnum(), \
                        "Ranges only supported on alphanumeric chars"
                    assert last < next, "Ranges must be from low to high!"
                    ranges.append((last, next))
                    index += 1
                else:
                    assert False, "Cannot have a range with an escaped character!"
            case '\\':
                assert index != len(inside) - 1
                escaped = inside[index+1]
                match escaped:
                    case 's':
                        chars += _whitespace_chars
                    case 'd':
                        ranges += _num_ranges
                    case 'w':
                        ranges += _alpha_num_ranges
                        chars += _alpha_num_chars
                    case '[' | ']' | '(' | ')' | '{' | '}' | '^' | '\\':
                        chars.append(escaped)
                    case _:
                        assert False, f"Unsupported escaped character: {escaped}"
                index += 1
            case _:
                chars.append(inside[index])
        index += 1
    return (CharSet(ranges, chars, inverted), end)

def parse(regex: str) -> Construction:
    '''
    Parse an AST for a regex from a string.
    '''
    #print(f"Parsing Regex from '{regex}'")
    instructions: list[Construction] = []
    index = 0

    # Loop through the string
    while index < len(regex):
        #print("Next Char: " + regex[index])
        match regex[index]:
            case '(':
                end = find_closing_paren(regex[index:])
                assert end is not None, f"Unmatched opening parenthesis at {index} of '{regex}'"
                end = index + end
                instructions.append(parse(regex[index+1:end]))
                index=end
            case ')':
                assert False, f"Unmatched closing parenthesis at {index} of '{regex}'"
            case '?':
                last = instructions.pop()
                instructions.append(Option(last))
            case '*':
                last = instructions.pop()
                instructions.append(Any(last))
            case '+':
                last = instructions.pop()
                instructions.append(Some(last))
            case '|':
                assert len(instructions) != 0, \
                    f"Alternative with empty option at {index} of '{regex}'"
                first = Sequence(instructions) if len(instructions) > 1 else instructions[0]
                second = parse(regex[index+1:])
                return Alternatives(first, second)
            case '.':
                instructions.append(WildCard())
            case '\\':
                assert index+1 < len(regex), \
                    f"Escape character with nothing after it at {index} of '{regex}'"
                
                escaped = regex[index+1]
                match escaped:
                    case 's' | 'S':
                        instructions.append(CharSet([], _whitespace_chars, escaped.isupper()))
                    case 'd' | 'D':
                        instructions.append(CharSet(_num_ranges, [], escaped.isupper()))
                    case 'w' | 'W':
                        instructions.append(
                            CharSet(_alpha_num_ranges, _alpha_num_chars, escaped.isupper()))
                    case _:
                        instructions.append(Literal(escaped))
                index += 1
            case '{':
                min_count, max_count, end = parse_count(regex[index:])
                index += end
                inst = instructions.pop()
                # Compile the the count into a sequence where anything
                # between the min and max count is optional.
                instructions += [inst] * min_count
                optional_count = max_count - min_count
                if optional_count > 0:
                    instructions += [inst] * optional_count
            case '[':
                inst, chars_parsed = parse_charset(regex[index:])
                instructions.append(inst)
                index += chars_parsed
            case _:
                instructions.append(Literal(regex[index]))
        index += 1

    assert len(instructions) != 0, f"Could not parse regular expression from '{regex}'"
    if len(instructions) == 1:
        retVal =  instructions[0]
    else:
        retVal = Sequence(instructions)
    #print(f"Parsed {retVal}")
    return retVal

def compile(regex: str) -> str:
    '''
    Compile a regex from its string representation to "assembly code"
    '''
    parsed = parse(regex)
    code, _ = parsed.compile()
    return f"# regex: {regex}\n" + code + "match\n"

def main():
    if len(sys.argv) < 2 or len(sys.argv) > 3:
        print(f"Usage: {sys.argv[0]} <regex> [out-file]", file=sys.stderr)
        sys.exit(1)

    regex = compile(sys.argv[1])

    if len(sys.argv) == 2:
        print(regex)
    else:
        with open(sys.argv[2], "w") as f:
            f.write(regex)

if __name__ == '__main__':
    main()