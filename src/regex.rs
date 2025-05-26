pub mod bin;

#[derive(PartialEq, Eq, Debug)]
pub enum Instruction {
    Save(usize, bool),
    Branch{
        c_min: u8,
        c_max: u8,
        dest: usize,
        consume: bool,
        inverted: bool},
    Split(usize, usize),
}
