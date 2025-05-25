from . import instruction as inst
import copy

def search(s: str, regex: list[inst.Instruction]) -> str | None:
    matches = []
    execution_step(regex, s, {}, matches, 0, 0)
    longest_match: str | None = None

    for start, end in matches:
        if (longest_match is None) or len(longest_match) < end - start:
            longest_match = s[start:end]

    return longest_match

def execution_step(
        program: list[inst.Instruction],
        s: str,
        save_data: dict[int, int],
        matches: list[tuple[int, int]],
        pc: int,
        sc: int
):
    i = program[pc]
    next_pc = pc+1
    if isinstance(i, inst.Match):
        matches.append((save_data[0], save_data[1]))
        return
    elif isinstance(i, inst.Save):
        save_data[i.index] = sc
    elif isinstance(i, inst.Jump):
        next_pc = i.dest
    elif isinstance(i, inst.Split):
        old_data = copy.deepcopy(save_data)
        execution_step(program, s, save_data, matches, i.dest2, sc)
        save_data = old_data
        next_pc = i.dest1
    elif isinstance(i, inst.Compare):
        if sc >= len(s):
            return
        c = s[sc]
        if not (c >= i.escaped_char1 and c <= i.escaped_char2):
            return
        sc += 1
    elif isinstance(i, inst.InvCompare):
        if sc >= len(s):
            return
        c = s[sc]
        if c >= i.escaped_char1 and c <= i.escaped_char2:
            return
        sc += 1
    elif isinstance(i, inst.OptCompare):
        if sc >= len(s):
            return
        c = s[sc]
        if c >= i.escaped_char1 and c <= i.escaped_char2:
            next_pc = i.dest
    else:
        raise AssertionError(f"{i} is not a recognized instruction!")

    execution_step(program, s, save_data, matches, next_pc, sc)
