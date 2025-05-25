#!/bin/env python3

import sys

import compiler.compiler as compiler

def main():
    if len(sys.argv) < 2 or len(sys.argv) > 3:
        print(f"Usage: {sys.argv[0]} <regex> [out-file]", file=sys.stderr)
        sys.exit(1)

    regex = compiler.compile_wrapper(sys.argv[1])

    if len(sys.argv) == 2:
        print(regex)
    else:
        with open(sys.argv[2], "w") as f:
            f.write(regex)

if __name__ == '__main__':
    main()