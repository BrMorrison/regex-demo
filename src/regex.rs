pub mod bin;

#[derive(PartialEq, Eq, Debug)]
pub enum Instruction {
    Match,
    Save(usize),
    Compare(u8, u8, bool),
    Branch(u8, u8, usize),
    Jump(usize),
    Split(usize, usize),
}
