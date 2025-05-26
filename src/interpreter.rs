mod thread;
use crate::regex::Instruction;
use crate::interpreter::thread::{ThreadList, ThreadGroup};
use std::mem;

struct Executor<'a> {
    program: &'a[Instruction],
}

impl <'a> Executor<'a> {
    fn new(prog: &'a[Instruction]) -> Self {
        Executor {
            program: prog,
        }
    }

    fn _execution_step(
            &self,
            current_threads: &mut ThreadList,
            temp_threads: &mut ThreadList,
            next_threads: &mut ThreadList,
            char_index: usize,
            input_char: u8
        ) -> Vec<(usize, usize)> {
        let mut consume_and_step = |pc: usize, thread_group: ThreadGroup| {
            next_threads.add_thread(pc, thread_group);
        };
        let mut step_execution = |pc: usize, thread_group: ThreadGroup| {
            temp_threads.add_thread(pc, thread_group);
        };
        let mut matches = Vec::new();
        for mut thread_group in current_threads.iter_mut() {
            let pc = thread_group.pc;
            match self.program[thread_group.pc] {
                Instruction::Match => {
                    let mut tmp_matches = thread_group.get_match_data(0);
                    matches.append(&mut tmp_matches)
                }
                Instruction::Save(dest) => {
                    thread_group.save(dest, char_index);
                    step_execution(pc + 1, thread_group);
                }

                Instruction::Compare(c_min, c_max, inverted) => {
                    let in_range = c_min <= input_char && input_char <= c_max;
                    if in_range != inverted{
                        consume_and_step(pc + 1, thread_group);
                    }
                }
                Instruction::Branch(c_min, c_max, new_pc) => {
                    if c_min <= input_char && input_char <= c_max {
                        step_execution(new_pc, thread_group);
                    } else {
                        step_execution(pc + 1, thread_group);
                    }
                }

                Instruction::Jump(new_pc) => step_execution(new_pc, thread_group),
                Instruction::Split(pc1, pc2) => {
                    step_execution(pc1, thread_group.clone());
                    step_execution(pc2, thread_group);
                }
            }
        }
        matches
    }

    fn execution_step(&mut self, current_threads: &mut ThreadList, char_index: usize, input_char: u8) -> Vec<(usize, usize)> {
        let mut temp_threads = ThreadList::new(self.program.len());
        let mut next_threads = ThreadList::new(self.program.len());
        let mut matches = Vec::new();

        while !current_threads.is_empty() {
            matches.append(&mut self._execution_step(current_threads, &mut temp_threads, &mut next_threads, char_index, input_char));
            current_threads.clear();
            mem::swap(current_threads, &mut temp_threads);
        }

        // Swap the next threads into current.
        current_threads.clear();
        mem::swap( current_threads, &mut next_threads);

        matches
    }

    fn run(&mut self, current_threads: &mut ThreadList, input: &'a str) -> Option<(usize, usize)> {
        let mut all_matches = Vec::new();

        for (char_index, input_char) in input.chars().enumerate() {
            let char_u8 = if input_char.is_ascii() {
                let mut char_buf: [u8; 1] = [0; 1];
                input_char.encode_utf8(& mut char_buf);
                char_buf[0]
            } else {
                // If it's unicode, send an invalid byte (that's not 0xFF)
                0xFE
            };

            all_matches.append(&mut self.execution_step(current_threads, char_index, char_u8));
        }

        // Run one final execution step in case there are any threads on a `match`
        all_matches.append(&mut self.execution_step(current_threads, input.len(), 0));

        let longer_match = |wrapped_match1: Option<(usize, usize)>, match2: &(usize, usize)| -> Option<(usize, usize)> {
            if let Some(match1) = wrapped_match1 {
                if match1.1 - match1.0 > match2.1 - match2.0 {
                    return wrapped_match1;
                }
            }
            Some(*match2)
        };

        all_matches.iter().fold(None, longer_match)

    }
}

pub fn search(prog: &[Instruction], input: &str) -> Option<(usize, usize)> {
    let mut executor = Executor::new(prog);
    let mut current_threads = ThreadList::new(prog.len());
    current_threads.add_thread(0, ThreadGroup::new(0));
    executor.run(&mut current_threads, input)
}
