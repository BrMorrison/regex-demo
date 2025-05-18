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
    Char(char),
    Jump(usize),
    Split(usize, usize),
    Charset(bool, Vec<(char, char)>, Vec<char>),
    WildCard,
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

fn parse_wildcard(args: &str) -> Result<Instruction, String> {
    if args != "" {
        return Err(format!("`any` expects 0 arguments, but was provided: {}", args));
    }
    Ok(Instruction::WildCard)
}

fn parse_charset(inverted: bool, args: &str) -> Result<Instruction, String> {
    // charset [<range-low>,<range-high>]... [<char>]...
    let mut ranges = Vec::new();
    let mut chars = Vec::new();
    for item in args.split(' ') {
        match item.split_once(',') {
            // If split once returns `None`, then there's no comma and this isn't a range
            None => {
                let c = read_escaped_char(item)?;
                chars.push(c);
            }
            Some((s_min, s_max)) => {
                let c_min = read_escaped_char(s_min)?;
                let c_max = read_escaped_char(s_max)?;
                if c_max < c_min {
                    return Err(format!("Invalid range {c_max} is less than {c_min}"));
                }
                ranges.push((c_min, c_max))
            } 
        }
    }
    Ok(Instruction::Charset(inverted, ranges, chars))
}

fn parse_char(args: &str) -> Result<Instruction, String> {
    // char <char>
    let c = read_escaped_char(args)?;
    Ok(Instruction::Char(c))
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
            "match" => parse_match(remainder),
            "any" => parse_wildcard(remainder),
            "charset" => parse_charset(false, remainder),
            "icharset" => parse_charset(true, remainder),
            "char" => parse_char(remainder),
            "jump" => parse_jump(remainder),
            "split" => parse_split(remainder),
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
