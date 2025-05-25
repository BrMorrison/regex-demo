from dataclasses import dataclass

"""
Instructions:
- Match
- Save <index>
- Jump <dest>
- Split <dest1> <dest2>
- Compare <char1> <char2>
- InvCompare <char1> <char2>
- OptCompare <char1> <char2> <dest>
"""

class Instruction:
    """
    Base class for regex instructions
    """
    def code(self) -> str:
        raise AssertionError("Base class should not be used")

@dataclass
class Match(Instruction):
    """
    Execution reaching this point means the input matches the regex.
    """
    def code(self) -> str:
        return "Match"

@dataclass
class Save(Instruction):
    """
    Saves the location in the input where a match begins or ends.
    """
    index: int
    def code(self) -> str:
        return f"Save {self.index}"

@dataclass
class Jump(Instruction):
    """
    Jump to a different location in the program.
    """
    dest: int
    def code(self) -> str:
        return f"Jump {self.dest}"

@dataclass
class Split(Instruction):
    """
    Continue execution from two different locations in the program.
    """
    dest1: int
    dest2: int
    def code(self) -> str:
        return f"Split {self.dest1} {self.dest2}"

@dataclass
class Compare(Instruction):
    """
    Consumes the current input character if it's within a given range and fails otherwise.
    """
    escaped_char1: str
    escaped_char2: str
    def code(self) -> str:
        return f"Compare {self.escaped_char1} {self.escaped_char2}"

@dataclass
class InvCompare(Instruction):
    """
    Consumes the current input character if it's not within a given range and fails otherwise.
    """
    escaped_char1: str
    escaped_char2: str
    def code(self) -> str:
        return f"InvCompare {self.escaped_char1} {self.escaped_char2}"

@dataclass
class OptCompare(Instruction):
    """
    Jump to a given destination if the current input character is within a given range.
    """
    escaped_char1: str
    escaped_char2: str
    dest: int
    def code(self) -> str:
        return f"OptCompare {self.escaped_char1} {self.escaped_char2} {self.dest}"
