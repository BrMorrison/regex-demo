use std::fmt;
use std::fs;
use std::error::Error;

#[derive(Debug)]
struct ParseError {
    file_path: String,
    line_number: usize,
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, "Error parsing regex from {} at line {}: {}",
            self.file_path, self.line_number, self.message)
    }
}

impl Error for ParseError {}

#[derive(PartialEq, Eq)]
pub enum Instruction {
    Match,
    Die,
    Consume,
    Char(char, bool),
    CharOption(char, usize),
    Range(char, char, bool),
    RangeOption(char, char, usize),
    Jump(usize),
    Split(usize, usize),
}

fn read_escaped_char(escaped: &str) -> Result<char, String> {
    let (prefix, encoded) = escaped.split_at(1);

    // If it's not an escaped character, just return it.
    if prefix != "%" {
        return match escaped.chars().next() {
            Some(c) => Ok(c),
            None => Err(format!("Could not read a character from '{}'", escaped))
        }
    }

    // Parse the escaped character
    match encoded.parse::<u32>() {
        Err(e) => Err(format!("Error reading escaped character {e}")),
        Ok(n) => match std::char::from_u32(n) {
            Some(c) => Ok(c),
            None => Err(format!("{n} is not a valid unicode character!")),
        }
    }
}

fn parse_match(args: &str) -> Result<Instruction, String> {
    if args != "" {
        return Err(format!("`match` expects 0 arguments, but was provided: {}", args));
    }
    Ok(Instruction::Match)
}

fn parse_die(args: &str) -> Result<Instruction, String> {
    if args != "" {
        return Err(format!("`die` expects 0 arguments, but was provided: {}", args));
    }
    Ok(Instruction::Die)
}

fn parse_wildcard(args: &str) -> Result<Instruction, String> {
    if args != "" {
        return Err(format!("`consume` expects 0 arguments, but was provided: {}", args));
    }
    Ok(Instruction::Consume)
}

fn parse_char(args: &str, inverted: bool) -> Result<Instruction, String> {
    // [i]char <char>
    let c = read_escaped_char(args)?;
    Ok(Instruction::Char(c, inverted))
}

fn parse_optional_char(args: &str) -> Result<Instruction, String> {
    // ochar <char> <pc>
    let (c, match_dest) = args.split_once(' ').unwrap();
    let c = read_escaped_char(c)?;

    let match_dest = match_dest.parse::<usize>();
    match match_dest {
        Ok(dest) => Ok(Instruction::CharOption(c, dest)),
        Err(err) => Err(format!("Failed to parse option destination with error: {err}")),
    }
}

fn parse_range(args: &str, inverted: bool) -> Result<Instruction, String> {
    // [i]range <min> <max>
    let args: Vec<&str> = args.split(' ').collect();
    if args.len() != 2 {
        return Err(format!("`range` expects 2 argument, but was provided {}: {:?}", args.len(), args));
    }
    let min_char = read_escaped_char(args[0])?;
    let max_char = read_escaped_char(args[1])?;
    if max_char < min_char {
        return Err(format!("Invalid range: {max_char} is less than {min_char}"));
    }

    Ok(Instruction::Range(min_char, max_char, inverted))
}

fn parse_optional_range(args: &str) -> Result<Instruction, String> {
    // optRange <min> <max> <pc>
    let args: Vec<&str> = args.split(' ').collect();
    if args.len() != 3 {
        return Err(format!("`range` expects 3 argument, but was provided {}: {:?}", args.len(), args));
    }
    let min_char = read_escaped_char(args[0])?;
    let max_char = read_escaped_char(args[1])?;
    if max_char < min_char {
        return Err(format!("Invalid range: {max_char} is less than {min_char}"));
    }

    let match_dest = args[2].parse::<usize>();
    match match_dest {
        Ok(dest) => Ok(Instruction::RangeOption(min_char, max_char, dest)),
        Err(err) => Err(format!("Failed to parse range destination with error: {err}")),
    }
}

fn parse_jump(args: &str) -> Result<Instruction, String> {
    // jump <dest>
    let dest = args.parse::<usize>();
    match dest {
        Ok(dest) => Ok(Instruction::Jump(dest)),
        Err(err) => Err(format!("Failed to parse jump destination with error: {err}")),
    }
}

fn parse_split(args: &str) -> Result<Instruction, String> {
    let args: Vec<&str> = args.split(' ').collect();
    if args.len() != 2 {
        return Err(format!("`split` expects 2 argument, but was provided {}: {:?}", args.len(), args));
    }
    let dest1 = args[0].parse::<usize>();
    let dest2 = args[1].parse::<usize>();
    match (dest1, dest2) {
        (Ok(dest1), Ok(dest2)) => Ok(Instruction::Split(dest1, dest2)),
        (Err(err), _) => Err(format!("Failed to parse first split destination with error: {err}")),
        (_, Err(err)) => Err(format!("Failed to parse second split destination with error: {err}")),
    }
}

pub fn parse_regex(file_path: &str) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let mut program = Vec::new();
    let contents = fs::read_to_string(file_path)?;
    for (line_number, line) in contents.lines().enumerate() {
        
        let maybe_opcode: Option<&str> = line.trim().split(' ').next();

        // Make sure there was something meaningful on the line.
        if maybe_opcode == None || maybe_opcode == Some("") {
            continue;
        }

        let opcode = maybe_opcode.unwrap();
        // Skip it if it's a comment.
        if opcode.starts_with("#") {
            continue;
        }

        let remainder = line.strip_prefix(opcode).unwrap().trim();

        let instruction = match opcode {
            "match"    => parse_match(remainder),
            "die"      => parse_die(remainder),
            "consume"  => parse_wildcard(remainder),
            "char"     => parse_char(remainder, false),
            "invChar"  => parse_char(remainder, true),
            "optChar"  => parse_optional_char(remainder),
            "range"    => parse_range(remainder, false),
            "invRange" => parse_range(remainder, true),
            "optRange" => parse_optional_range(remainder),
            "jump"     => parse_jump(remainder),
            "split"    => parse_split(remainder),
            _ => Err(format!("Unrecognized opcode `{}`", opcode)),
        };

        match instruction {
            Ok(instruction) => program.push(instruction),
            Err(msg) => {
                return Err(Box::new(ParseError {
                    file_path: file_path.to_string(),
                    line_number: line_number,
                    message: msg,
                }));
            }
        }
    }

    // Make sure the program ends in a `match` instruction.
    let last_instruction = program.last();
    if last_instruction != Some(&Instruction::Match) {
        return Err(Box::new(ParseError {
            file_path: file_path.to_string(),
            line_number: contents.lines().count(),
            message: "Program must end with a `match`".to_string(),
        }));
    }

    Ok(program)
}
