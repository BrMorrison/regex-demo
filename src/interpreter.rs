use std::mem;
use crate::regex::Instruction;

struct Thread {
    pc: usize,
}

#[derive(Debug)]
struct ThreadList {
    size: usize,
    threads: Vec<u64>,
}

struct ThreadListIter<'a> {
    thread_list: &'a ThreadList,
    index: usize,
}

struct Executor<'a> {
    program: &'a[Instruction],
    current_threads: ThreadList,
}

impl Thread {
    fn new(pc: usize) -> Self {
        Thread {
            pc: pc
        }
    }
}

impl ThreadList {
    fn new(capacity: usize) -> Self {
        // How many u64s do we need to have a bit for each possible program address?
        // The ceiling of capacity/64
        let num_elems = capacity.div_ceil(64);
        ThreadList { size: capacity, threads: vec![0; num_elems] }
    }

    fn add_thread(&mut self, pc: usize) {
        if pc > self.size {
            return;
        }
        // Strength-reduction should deal with these multiplications/divisions by a power of 2.
        let threads_index = pc / 64;
        let thread_bit_index = pc - (threads_index * 64);
        self.threads[threads_index] |= 1 << thread_bit_index;
    }

    fn clear(&mut self) {
        for elem in self.threads.iter_mut() {
            *elem = 0;
        }
    }

    fn iter(&self) -> ThreadListIter {
        ThreadListIter {
            thread_list: self,
            index: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.threads.iter().all(|val| { *val == 0 })
    }
}

impl <'a> Iterator for ThreadListIter<'a> {
    type Item = Thread;

    fn next(&mut self) -> Option<Self::Item> {
        let mut threads_index = self.index / 64;
        while self.index <= self.thread_list.size {
            // Get the u64 from the threadlist that contains our current index.
            let thread_bit_index = self.index - (threads_index * 64);
            let elem = self.thread_list.threads[threads_index];

            // Clear out all the bits below our index and see if we have any set bits remaining.
            let remainder = elem >> thread_bit_index;
            if remainder == 0 {
                // If we've run out of bits in this block, move on to the next one.
                threads_index += 1;
                self.index = threads_index * 64;
                continue;
            }

            let next_bit_index = remainder.trailing_zeros() + thread_bit_index as u32;
            let next_index = next_bit_index as usize + (threads_index * 64);
            self.index = next_index + 1;
            return Some(Thread::new(next_index));
        }

        None
    }
}

impl <'a> Executor<'a> {
    fn new(prog: &'a[Instruction]) -> Self {
        // Worst case number of threads is the number of instructions in the program.
        let prog_size = prog.len();

        Executor {
            program: prog,
            current_threads: ThreadList::new(prog_size),
        }
    }

    fn _execution_step(&self, temp_threads: &mut ThreadList, next_threads: &mut ThreadList, input_char: Option<char>) -> bool {
        let mut consume_and_step = |pc: usize| { next_threads.add_thread(pc); };
        let mut step_execution = |pc: usize| { temp_threads.add_thread(pc); };
        for thread in self.current_threads.iter() {
            match (&self.program[thread.pc], &input_char) {
                (Instruction::Match, _) => return true,
                (Instruction::Die, _) => (), // Do nothing and let the thread die.
                (Instruction::Consume, Some(_)) => consume_and_step(thread.pc + 1),
                (Instruction::Consume, None) => (),

                (Instruction::Char(_, _), None) => (),
                (Instruction::Char(c, false), Some(input_c)) => if input_c == c {
                    consume_and_step(thread.pc + 1);
                }
                (Instruction::Char(c, true), Some(input_c)) => if input_c != c {
                    // The inverted matchers don't consume input characters
                    step_execution(thread.pc + 1);
                }

                (Instruction::CharOption(_, _), None) => (),
                (Instruction::CharOption(c, pc), _) => {
                    if input_char == Some(*c) {
                        consume_and_step(*pc);
                    } else {
                        step_execution(thread.pc + 1);
                    }
                }

                (Instruction::Range(_, _, _), None) => (),
                (Instruction::Range(c_min, c_max, false), Some(c)) => {
                    if c_min <= c && c <= c_max {
                        consume_and_step(thread.pc + 1);
                    }
                }
                (Instruction::Range(c_min, c_max, true), Some(c)) => {
                    if c < c_min || c_max < c {
                        step_execution(thread.pc + 1);
                    }
                }

                (Instruction::RangeOption(_, _, _), None) => (),
                (Instruction::RangeOption(c_min, c_max, pc), Some(c)) => {
                    if c_min <= c && c <= c_max {
                        consume_and_step(*pc);
                    } else {
                        step_execution(thread.pc + 1);
                    }
                }

                (Instruction::Jump(pc), _) => step_execution(*pc),
                (Instruction::Split(pc1, pc2), _) => {
                    step_execution(*pc1);
                    step_execution(*pc2);
                }
            }
        }
        false
    }

    fn execution_step(&mut self, input_char: Option<char>) -> bool {
        // For each character in the string, try and match the regex starting at that
        // character. This allows for partial matches in the string.
        if let Some(_) = input_char {
            self.current_threads.add_thread(0);
        }

        let mut temp_threads = ThreadList::new(self.program.len());
        let mut next_threads = ThreadList::new(self.program.len());

        while !self.current_threads.is_empty() {
            if self._execution_step(&mut temp_threads, &mut next_threads, input_char) {
                return true;
            }
            self.current_threads.clear();
            mem::swap(&mut self.current_threads, &mut temp_threads);
        }

        // Swap the new threads into current.
        self.current_threads.clear();
        mem::swap(&mut self.current_threads, &mut next_threads);
        false
    }

    fn run(&mut self, input: &str) -> bool {
        for input_char in input.chars() {
            if self.execution_step(Some(input_char)) {
                return true;
            }
        }

        // Run one final execution step in case there are any threads on a `match`
        self.execution_step(None)
    }
}

pub fn search(prog: &[Instruction], input: &str) -> bool {
    let mut executor = Executor::new(prog);
    executor.run(input)
}
