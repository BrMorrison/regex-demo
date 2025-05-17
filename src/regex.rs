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
}

fn parse_match(args: &[&str]) -> Result<Instruction, String> {
    if args.len() != 0 {
        return Err(format!("`match` expects 0 arguments, but was provided {}: {:?}", args.len(), args));
    }
    Ok(Instruction::Match)
}

fn parse_char(args: &[&str]) -> Result<Instruction, String> {
    if args.len() != 1 {
        return Err(format!("`char` expects 1 argument, but was provided {}: {:?}", args.len(), args));
    }

    let arg = args[0];
    if arg.chars().count() != 1 {
        return Err(format!("The argument `char` must be one character, but encountered {}", arg));
    }

    Ok(Instruction::Char(arg.chars().next().unwrap()))
}

fn parse_jump(args: &[&str]) -> Result<Instruction, String> {
    if args.len() != 1 {
        return Err(format!("`jump` expects 1 argument, but was provided {}: {:?}", args.len(), args));
    }

    let dest = args[0].parse::<usize>();
    match dest {
        Ok(dest) => Ok(Instruction::Jump(dest)),
        Err(err) => Err(format!("Failed to parse jump destination with error: {err}")),
    }
}

fn parse_split(args: &[&str]) -> Result<Instruction, String> {
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
        let words: Vec<&str> = line.trim().split(' ').collect();

        // Make sure there was something on the line
        if words.len() == 0  || words[0] == "" {
            continue;
        }

        let opcode = words[0];
        let instruction = match opcode {
            "match" => parse_match(&words[1..]),
            "char" => parse_char(&words[1..]),
            "jump" => parse_jump(&words[1..]),
            "split" => parse_split(&words[1..]),
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
