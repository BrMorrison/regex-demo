use std::collections::LinkedList;
use std::mem;
use std::slice;
use std::vec;

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


type StoredThreadData = (usize, LinkedList<ThreadData>);

pub struct ThreadList {
    threads: Vec<StoredThreadData>,
}

pub struct ThreadListIterMut<'a> {
    iter: slice::IterMut<'a, StoredThreadData>,
}

impl ThreadList {
    pub fn new(capacity: usize) -> Self {
        ThreadList { threads: Vec::with_capacity(capacity) }
    }

    pub fn add_thread(&mut self, pc: usize, mut thread_data: ThreadGroup) {
        if let Some((_, data)) = self.threads.iter_mut().find(|(stored_pc, _)| {*stored_pc == pc}) {
            data.append(&mut thread_data.data);
        } else {
            self.threads.push((pc, thread_data.data));
        }
    }

    pub fn clear(&mut self) {
        self.threads.clear()
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
    data: LinkedList<ThreadData>,
}

impl ThreadGroup {
    pub fn new(pc: usize) -> Self {
        ThreadGroup {
            pc: pc,
            data: LinkedList::from([ThreadData::new()]),
        }
    }

    pub fn save(&mut self, match_index: usize, char_index: usize) {
        for thread_data in self.data.iter_mut() {
            thread_data.match_indices[match_index] = char_index;
        }
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
        match self.iter.next() {
            None => None,
            Some((pc, data)) => Some(ThreadGroup {
                pc: *pc,
                data: mem::take(data),
            }),
        }
    }
}