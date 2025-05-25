#!/bin/env python3

from syntax import *
from parser import *
import sys

def compile(regex: str) -> str:
    '''
    Compile a regex from its string representation to "assembly code"
    '''
    # Wrap the regex in a group to allow for extraction of the match.
    parsed = Group(0, parse(regex))

    # If the regex isn't trying to match from the start of the string, then its equivalent to
    # matching anything (.*) before the provided regex.
    if regex[0] != '$':
        prefix = parse('.*')
        parsed = Sequence([prefix, parsed])

    code, _ = parsed.compile()
    return f"# regex: {regex}\n" + code + "Match\n"

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