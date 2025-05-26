use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;

use crate::regex::Instruction;

const OPCODE_MASK: u32 = 0xC000_0000;
const SAVE_INDEX_MASK: u32 = 0x003F_0000;
const INVERTED_MASK: u32 = 0x2000_0000;
const CONSUME_MASK: u32 = 0x1000_0000;
const MATCH_MASK: u32 = INVERTED_MASK;
const DEST_MASK: u32 = 0x0FFF_0000;
const DEST2_MASK: u32 = 0x0000_FFF0;
const CHAR_MIN_MASK: u32 = 0x0000_FF00;
const CHAR_MAX_MASK: u32 = 0x0000_00FF;

const OPCODE_SHIFT: u32 = 30;
const SAVE_INDEX_SHIFT: u32 = 16;
const DEST_SHIFT: u32 = 16;
const DEST2_SHIFT: u32 = 4;
const CHAR_MIN_SHIFT: u32 = 8;
const CHAR_MAX_SHIFT: u32 = 0;


pub fn parse_bin(path: &str) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let mut f = File::open(path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    
    let mut instructions = Vec::new();

    // Each instruction is 32 bits
    for chunk in buf.chunks_exact(4) {
        let inst = parse_instruction(chunk)?;
        instructions.push(inst);
    }

    Ok(instructions)
}

#[derive(Debug)]
struct ParseError {
    instruction: u32,
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, "Error parsing regex from 0x{:#10x}: {}",
            self.instruction, self.message)
    }
}

impl Error for ParseError {}

fn parse_instruction(bytes: &[u8]) -> Result<Instruction, Box<dyn Error>> {

    let (b1, b2, b3, b4) = (bytes[0] as u32, bytes[1] as u32, bytes[2] as u32, bytes[3] as u32);
    let combined = b1 << 24 | b2 << 16 | b3 << 8 | b4;

    let opcode = (combined & OPCODE_MASK) >> OPCODE_SHIFT;

    match opcode {
        0b00 => Ok(parse_branch(combined)),
        0b01 => Ok(parse_split(combined)),
        0b10 => Ok(parse_save(combined)),
        _ => Err(Box::new(ParseError {
                instruction: combined,
                message: format!("Did not recognize opcode {:#05b}", opcode)})),
    }
}


fn parse_split(instruction: u32) -> Instruction {
    let dest1 = (instruction & DEST_MASK) >> DEST_SHIFT;
    let dest2 = (instruction & DEST2_MASK) >> DEST2_SHIFT;
    Instruction::Split(dest1 as usize, dest2 as usize)
}

fn parse_branch(instruction: u32) -> Instruction {
    let inverted = (instruction & INVERTED_MASK) != 0;
    let consume = (instruction & CONSUME_MASK) != 0;
    let dest = (instruction & DEST_MASK) >> DEST_SHIFT;
    let char_min = (instruction & CHAR_MIN_MASK) >> CHAR_MIN_SHIFT;
    let char_max = (instruction & CHAR_MAX_MASK) >> CHAR_MAX_SHIFT;
    Instruction::Branch{
        c_min: char_min as u8,
        c_max: char_max as u8,
        dest: dest as usize,
        consume: consume,
        inverted: inverted}
}

fn parse_save(instruction: u32) -> Instruction {
    let is_match = (instruction & MATCH_MASK) != 0;
    let index = (instruction & SAVE_INDEX_MASK) >> SAVE_INDEX_SHIFT;
    Instruction::Save(index as usize, is_match)
}
