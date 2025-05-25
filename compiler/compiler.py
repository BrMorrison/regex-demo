from . import syntax, parser, code_gen, instruction
#import parser
#import code_gen
#import instruction


def compile_regex(regex: str) -> list[instruction.Instruction]:
    # Wrap the regex in a group to allow for extraction of the match.
    parsed = syntax.Group(0, parser.parse(regex))

    # If the regex isn't trying to match from the start of the string, then its equivalent to
    # matching anything (.*) before the provided regex.
    if regex[0] != '$':
        prefix = parser.parse('.*')
        parsed = syntax.Sequence([prefix, parsed])
    
    code = code_gen.compile(parsed)
    return code

def compile_wrapper(regex: str) -> str:
    '''
    Compile a regex from its string representation to "assembly code"
    '''
    code = compile_regex(regex)
    code_text = '\n'.join(map(lambda inst: inst.code(), code))
    return f"# regex: {regex}\n" + code_text

def main():
    if len(sys.argv) < 2 or len(sys.argv) > 3:
        print(f"Usage: {sys.argv[0]} <regex> [out-file]", file=sys.stderr)
        sys.exit(1)

    regex = compile_wrapper(sys.argv[1])

    if len(sys.argv) == 2:
        print(regex)
    else:
        with open(sys.argv[2], "w") as f:
            f.write(regex)

if __name__ == '__main__':
    import sys
    main()
else:
    from . import code_gen, instruction