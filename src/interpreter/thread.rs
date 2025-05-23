use std::collections::hash_map;
use std::collections::HashMap;
use std::collections::HashSet;
use std::mem;

#[derive(PartialEq, Eq, Clone, Hash)]
struct ThreadData {
    // TODO: Can we make this Vec<(usize, usize)> since the indices always come in pairs?
    match_indices: Vec<usize>,
}

impl ThreadData {
    fn new() -> Self {
        ThreadData {
            // For now, we know that there will only ever be 2 indices (start and end), but this
            // won't be true if we ever support submatches.
            match_indices: vec![0, 0],
        }
    }
}

pub struct ThreadList {
    threads: HashMap<usize, HashSet<ThreadData>>,
}

pub struct ThreadListIterMut<'a> {
    iter: hash_map::IterMut<'a, usize, HashSet<ThreadData>>,
}

impl ThreadList {
    pub fn new(capacity: usize) -> Self {
        ThreadList { threads: HashMap::with_capacity(capacity) }
    }

    pub fn add_thread(&mut self, pc: usize, mut thread_data: ThreadGroup) {
        match self.threads.get_mut(&pc) {
            // If there is already thread data, combine it with the new data
            Some(data) => {
                for val in thread_data.data.drain() {
                    data.insert(val);
                }
            },
            None => {self.threads.insert(pc, thread_data.data);}
        }
    }

    pub fn clear(&mut self) {
        self.threads.clear();
    }

    pub fn iter_mut(&mut self) -> ThreadListIterMut {
        ThreadListIterMut { iter: self.threads.iter_mut() }
    }

    pub fn is_empty(&self) -> bool {
        self.threads.is_empty()
    }

}

/// A group of threads that are all at the same execution point in the program.
#[derive(Clone)]
pub struct ThreadGroup {
    pub pc: usize,
    data: HashSet<ThreadData>,
}

impl ThreadGroup {
    pub fn new(pc: usize) -> Self {
        ThreadGroup {
            pc: pc,
            data: HashSet::from([ThreadData::new()]),
        }
    }

    pub fn save(&mut self, match_index: usize, char_index: usize) {
        self.data = self.data.drain()
            .map(|mut data| { data.match_indices[match_index] = char_index; data })
            .collect();
    }

    pub fn get_match_data(&self, match_index: usize) -> Vec<(usize, usize)> {
        let mut char_indices = Vec::with_capacity(self.data.len());
        for data in self.data.iter() {
            char_indices.push((data.match_indices[match_index*2], data.match_indices[match_index*2+1]));
        }
        char_indices
    }

}

impl <'a> Iterator for ThreadListIterMut<'a> {
    type Item = ThreadGroup;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(pc, thread_data)| {
            ThreadGroup { pc: *pc, data: mem::take(thread_data) }
        })
    }
}