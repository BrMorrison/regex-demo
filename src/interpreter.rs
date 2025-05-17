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
    new_threads: ThreadList,
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
            new_threads: ThreadList::new(prog_size),
        }
    }

    fn execution_step(&mut self, input_char: Option<char>) -> bool {
        // For each character in the string, try and match the regex starting at that
        // character. This allows for partial matches in the string.
        if let Some(_) = input_char {
            self.current_threads.add_thread(0);
        }

        let mut temp_threads = ThreadList::new(self.program.len());
        while !self.current_threads.threads.is_empty() {
            for thread in self.current_threads.threads.iter() {
                match self.program[thread.pc] {
                    Instruction::Match => return true,
                    Instruction::Char(c) => if input_char == Some(c) {
                        self.new_threads.add_thread(thread.pc + 1);
                    }
                    Instruction::Jump(pc) => temp_threads.add_thread(pc),
                    Instruction::Split(pc1, pc2) => {
                        temp_threads.add_thread(pc1);
                        temp_threads.add_thread(pc2);
                    }
                }
            }
            self.current_threads.threads.clear();
            mem::swap(&mut self.current_threads, &mut temp_threads);
        }

        // Swap the new threads into current.
        self.current_threads.threads.clear();
        mem::swap(&mut self.current_threads, &mut self.new_threads);
        self.new_threads = ThreadList::new(self.program.len());
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
