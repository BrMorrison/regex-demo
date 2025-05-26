use std::collections::LinkedList;
use std::iter::Enumerate;
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

pub struct ThreadList {
    threads: Vec<Option<LinkedList<ThreadData>>>,
}

pub struct ThreadListIterMut<'a> {
    iter: Enumerate<slice::IterMut<'a, Option<LinkedList<ThreadData>>>>,
}

impl ThreadList {
    pub fn new(capacity: usize) -> Self {
        ThreadList { threads: vec![None; capacity] }
    }

    pub fn add_thread(&mut self, pc: usize, mut thread_data: ThreadGroup) {
        if let Some(data) =  &mut self.threads[pc] {
            data.append(&mut thread_data.data);
        } else {
            let mut new_data = LinkedList::new();
            new_data.append(&mut thread_data.data);
            self.threads[pc] = Some(new_data)
        }
    }

    pub fn clear(&mut self) {
        for thread in self.threads.iter_mut() {
            *thread = None;
        }
    }

    pub fn iter_mut(&mut self) -> ThreadListIterMut {
        ThreadListIterMut { iter: self.threads.iter_mut().enumerate() }
    }

    pub fn is_empty(&self) -> bool {
        self.threads.iter().all(|t| { t.is_none() })
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
        loop {
            match self.iter.next() {
                None => return None,
                Some((_, None)) => continue,
                Some((pc, Some(data))) => {
                    
                    return Some(ThreadGroup {
                        pc: pc,
                        data: mem::take(data),
                    })
                },
            }
        }
    }
}