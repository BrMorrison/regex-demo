#!/bin/env python3

from dataclasses import dataclass
import sys

class Construction:
    def compile(self, pc: int=0) -> tuple[str, int]:
        return ('', pc)

@dataclass
class Match(Construction):
    def compile(self, pc: int=0) -> tuple[str, int]:
        return (f"match\n", pc+1)

@dataclass
class Literal(Construction):
    val: str

    def compile(self, pc: int=0) -> tuple[str, int]:
        return (f"char {self.val}\n", pc+1)

@dataclass
class Sequence(Construction):
    val: list[Construction]

    def compile(self, pc: int=0) -> tuple[str, int]:
        code = ''
        for val in self.val:
            tmp_code, pc = val.compile(pc)
            code += tmp_code
        return (code, pc)

@dataclass
class Alternatives(Construction):
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
    val: Construction

    def compile(self, pc: int=0) -> tuple[str, int]:
        """
            split L1, L2
        L1: code for val
        L2:
        """
        l1 = pc+1
        code, l2 = self.val.compile(l1)
        return (f"split {l1} {l2}\n" + code, l2)

@dataclass
class Some(Construction):
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


def find_closing_paren(s: str) -> int | None:
    assert s[0] == '('
    depth = 0
    for i, c in enumerate(s):
        match c:
            case '(':
                depth += 1
            case ')':
                depth -= 1
                if depth == 0:
                    return i
    return None

def _parse(regex: str) -> Construction:
    #print(f"Parsing regex from '{regex}'")
    instructions: list[Construction] = []
    index = 0
    while index < len(regex):
        match regex[index]:
            case '(':
                end = find_closing_paren(regex[index:])
                assert end is not None, f"Unmatched opening parenthesis at {index} of '{regex}'"
                #print(f"start:{index}, end:{end}, substr:'{regex[index+1:end+1]}'")
                instructions.append(_parse(regex[index+1:end+1]))
                index=end+1
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
                assert len(instructions) != 0, f"Alternative with empty option at {index} of '{regex}'"
                first = Sequence(instructions) if len(instructions) > 1 else instructions[0]
                second = _parse(regex[index+1:])
                return Alternatives(first, second)
            case '\\':
                assert index+1 < len(regex), f"Escape character with nothing after it at {index} of '{regex}'"
                index += 1
                instructions.append(Literal(regex[index]))
            case _:
                instructions.append(Literal(regex[index]))
        index += 1

    assert len(instructions) != 0, f"Could not parse regular expression from '{regex}'"
    if len(instructions) == 1:
        return instructions[0]
    else:
        return Sequence(instructions)

def parse(regex: str) -> Construction:
    parsed = _parse(regex)
    return Sequence([parsed, Match()])

def main():
    if len(sys.argv) < 2 or len(sys.argv) > 3:
        print(f"Usage: {sys.argv[0]} <regex> [out-file]", file=sys.stderr)
        sys.exit(1)

    parsed_regex = parse(sys.argv[1])
    #print(parsed_regex)
    #sys.exit(0)
    regex, _ = parsed_regex.compile()

    if len(sys.argv) == 2:
        print(regex)
    else:
        with open(sys.argv[2], "w") as f:
            f.write(regex)

if __name__ == '__main__':
    main()