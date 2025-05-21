use std::mem;
use crate::regex::Instruction;

struct Thread {
    pc: usize,
}

struct ThreadList {
    threads: Vec<Thread>,
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
        ThreadList { threads: Vec::with_capacity(capacity) }
    }

    fn add_thread(&mut self, pc: usize) {
        for thread in self.threads.iter() {
            if thread.pc == pc {
                return;
            }
        }
        self.threads.push(Thread::new(pc));
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
        for thread in self.current_threads.threads.iter() {
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
        while !self.current_threads.threads.is_empty() {
            if self._execution_step(&mut temp_threads, &mut next_threads, input_char) {
                return true;
            }
            self.current_threads.threads.clear();
            mem::swap(&mut self.current_threads, &mut temp_threads);
        }

        // Swap the new threads into current.
        self.current_threads.threads.clear();
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
