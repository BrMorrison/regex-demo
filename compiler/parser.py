from syntax import *

_whitespace_chars = ['\n', ' ', '\t', '\r', '\f', '\v']
_alpha_num_ranges = [('0','9'), ('A', 'Z'), ('a', 'z')]
_alpha_num_chars = ['_']
_num_ranges = [('0', '9')]

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
    return retVal