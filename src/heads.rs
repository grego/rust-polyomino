
// head of rows and columns
#[derive(Clone)]
pub struct Head {
    previous: usize,
    next: usize,
}

// pseudo-linked list of heads
#[derive(Clone)]
pub struct HeadList {
    terminal: usize, // blank, "terminal" head (should be the last node)
    data: Vec<Head>,
}

impl HeadList {
    pub fn new(size: usize) -> Self {
        HeadList {
            terminal: size,
            data: (0..=size)
                .map(|i| Head {
                    previous: if i != 0 { i - 1 } else { size },
                    next: if i != size { i + 1 } else { 0 },
                })
                .collect(),
        }
    }

    pub fn remove(&mut self, i: usize) {
        let previous = self.data[i].previous;
        let next = self.data[i].next;
        self.data[previous].next = next;
        self.data[next].previous = previous;
    }

    pub fn restore(&mut self, i: usize) {
        let previous = self.data[i].previous;
        let next = self.data[i].next;
        self.data[previous].next = i;
        self.data[next].previous = i;
    }

    pub fn iter<'a>(&'a self) -> HeadListIter<'a> {
        HeadListIter {
            list: self,
            current: self.terminal,
        }
    }
}

pub struct HeadListIter<'a> {
    list: &'a HeadList,
    current: usize,
}

impl<'a> Iterator for HeadListIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.list.data[self.current].next != self.list.terminal {
            self.current = self.list.data[self.current].next;
            Some(self.current)
        } else {
            None
        }
    }
}
