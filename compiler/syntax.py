from dataclasses import dataclass

class Construction:
    '''
    Base type for all regular expression grammar constructions.
    '''

@dataclass
class Literal(Construction):
    '''
    A single character literal to match.
    '''
    val: str

@dataclass
class Group(Construction):
    '''
    A subexpression that can be matched and have it's location extracted afterwards.
    Currently just supported for the top-level expression.
    '''
    expression_index: int
    expression: Construction

@dataclass
class WildCard(Construction):
    '''
    Matches any single character, ex: `.`
    '''

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


@dataclass
class Sequence(Construction):
    '''
    Matches a sequence of sub-expressions, ex: `abc`
    '''
    val: list[Construction]

@dataclass
class Alternatives(Construction):
    '''
    Matches one of multiple alternative sub-expressions, ex: `ab|cd`
    '''
    alt1: Construction
    alt2: Construction

@dataclass
class Option(Construction):
    '''
    Matches zero or one occurrences, ex: `a?`
    '''
    val: Construction

@dataclass
class Some(Construction):
    '''
    Matches one or more occurrences, ex: `a+`
    '''
    val: Construction

@dataclass
class Any(Construction):
    '''
    Matches zero or more occurrences, ex: `a*`
    '''
    val: Construction
